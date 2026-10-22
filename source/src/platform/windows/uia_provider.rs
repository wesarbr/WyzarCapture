//! Snapshot-backed UIA fragments. COM clients never borrow the live App or Host.
//! Actions enter a bounded queue and are applied only by the owning window thread.
#![allow(non_upper_case_globals)] // Preserve Windows SDK identifier spelling.
use crate::ui::app::{Action, App, Field, SliderKind, Toggle};
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use windows::{
    Win32::{
        Foundation::*,
        System::{Com::SAFEARRAY, Ole::*, Variant::*},
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
    core::{
        BOOL, BSTR, Error, HRESULT, IUnknown, IUnknownImpl, Interface, PCWSTR, Result, implement,
    },
};
pub const ACTION_MESSAGE: u32 = WM_APP + 41;
#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Button,
    Edit(String),
    Toggle(bool),
    Range(f64, f64, f64),
}
#[derive(Clone)]
struct Node {
    id: usize,
    key: String,
    name: String,
    rect: UiaRect,
    kind: Kind,
}
struct Model {
    hwnd: isize,
    alive: bool,
    active: bool,
    epoch: i32,
    context: String,
    next_id: usize,
    nodes: Vec<Node>,
    focus: Option<usize>,
    pending: VecDeque<Request>,
}
#[derive(Clone)]
pub struct Bridge(Arc<Mutex<Model>>);
#[derive(Clone)]
pub enum Command {
    Invoke,
    Focus,
    Text(String),
    Number(f64),
}
#[derive(Clone)]
pub struct Request {
    epoch: i32,
    index: usize,
    pub command: Command,
}
thread_local! { static ROOTS: RefCell<HashMap<isize, IRawElementProviderSimple>> = RefCell::new(HashMap::new()); }
fn unavailable() -> Error {
    HRESULT(UIA_E_ELEMENTNOTAVAILABLE as i32).into()
}
fn unsupported<T>() -> Result<T> {
    Err(HRESULT(UIA_E_NOTSUPPORTED as i32).into())
}
// The bindings use Result<Interface> for COM's nullable output. S_OK with no
// interface is represented by Error::empty(); UIA initializes nullable outputs.
fn no_element<T>() -> Result<T> {
    Err(Error::empty())
}
fn assign_ids(nodes: &mut [Node], previous: &[Node], context_changed: bool, next: &mut usize) {
    let mut identities: HashMap<&str, VecDeque<usize>> = HashMap::new();
    if !context_changed {
        for old in previous {
            identities.entry(&old.key).or_default().push_back(old.id);
        }
    }
    for node in nodes {
        node.id = identities
            .get_mut(node.key.as_str())
            .and_then(VecDeque::pop_front)
            .unwrap_or_else(|| {
                let id = *next;
                *next = next.wrapping_add(1).max(1);
                id
            });
    }
}
impl Bridge {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Model {
            hwnd: 0,
            alive: true,
            active: false,
            epoch: 1,
            context: String::new(),
            next_id: 1,
            nodes: vec![],
            focus: None,
            pending: VecDeque::new(),
        })))
    }
    pub fn register(&self, hwnd: HWND) {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).hwnd = hwnd.0 as isize;
        ROOTS.with(|roots| {
            roots
                .borrow_mut()
                .insert(hwnd.0 as isize, self.provider(None, 0));
        });
    }
    fn provider(&self, index: Option<usize>, epoch: i32) -> IRawElementProviderSimple {
        Provider {
            bridge: self.clone(),
            index,
            epoch,
        }
        .into()
    }
    pub fn suspend(&self) {
        let mut m = self.0.lock().unwrap_or_else(|e| e.into_inner());
        m.active = false;
        // Keep already accepted actions. The next complete snapshot invalidates
        // old-page epochs; ordinary value/focus changes must not drop a queue.
    }
    pub fn update(&self, app: &App, scale: f32, hwnd: HWND) {
        let mut origin = POINT::default();
        unsafe {
            let _ = windows::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut origin);
        }
        let mut nodes: Vec<_> = app
            .hits
            .iter()
            .map(|hit| {
                let kind = match &hit.action {
                    Action::Focus(f) => Kind::Edit(field_value(app, *f)),
                    Action::Toggle(t) => Kind::Toggle(toggle_value(app, *t)),
                    Action::Slider(t, _) => {
                        let clip = app.project.clips.get(app.project.selected);
                        match t {
                            SliderKind::Volume => Kind::Range(
                                clip.map(|c| c.volume as f64 * 100.).unwrap_or(100.),
                                0.,
                                150.,
                            ),
                            SliderKind::Speed => {
                                Kind::Range(clip.map(|c| c.speed).unwrap_or(1.), 0.25, 2.)
                            }
                        }
                    }
                    _ => Kind::Button,
                };
                let name = if matches!(hit.action, Action::Toggle(_)) {
                    if let Action::Toggle(t) = hit.action {
                        toggle_name(t).to_string()
                    } else {
                        hit.label.clone()
                    }
                } else if hit.label.is_empty() {
                    fallback_name(&hit.action).to_string()
                } else {
                    hit.label.clone()
                };
                Node {
                    id: 0,
                    key: format!("{:?}", hit.action),
                    name,
                    kind,
                    rect: UiaRect {
                        left: origin.x as f64 + (hit.rect.x * scale) as f64,
                        top: origin.y as f64 + (hit.rect.y * scale) as f64,
                        width: (hit.rect.w * scale) as f64,
                        height: (hit.rect.h * scale) as f64,
                    },
                }
            })
            .collect();
        let changed;
        let focus_changed;
        let epoch;
        let focus_id;
        let mut property_changes = vec![];
        {
            let mut m = self.0.lock().unwrap_or_else(|e| e.into_inner());
            let context = format!(
                "{:?}:{:?}:{:?}:{}",
                app.page,
                app.modal.as_ref().map(std::mem::discriminant),
                app.interface_mode(),
                app.category
            );
            let context_changed = m.context != context;
            changed = context_changed
                || m.nodes.len() != nodes.len()
                || m.nodes.iter().zip(&nodes).any(|(a, b)| a.key != b.key);
            if context_changed {
                m.epoch = m.epoch.wrapping_add(1).max(1);
                m.pending.clear();
            }
            let mut next = m.next_id;
            assign_ids(&mut nodes, &m.nodes, context_changed, &mut next);
            m.next_id = next;
            m.context = context;
            if !context_changed {
                for new in &nodes {
                    let Some(old) = m.nodes.iter().find(|old| old.id == new.id) else {
                        continue;
                    };
                    let index = new.id;
                    if old.name != new.name {
                        property_changes.push((
                            index,
                            UIA_NamePropertyId,
                            VARIANT::from(old.name.as_str()),
                            VARIANT::from(new.name.as_str()),
                        ));
                    }
                    match (&old.kind, &new.kind) {
                        (Kind::Range(a, ..), Kind::Range(b, ..)) if a != b => property_changes
                            .push((
                                index,
                                UIA_RangeValueValuePropertyId,
                                VARIANT::from(*a),
                                VARIANT::from(*b),
                            )),
                        (Kind::Edit(a), Kind::Edit(b)) if a != b => property_changes.push((
                            index,
                            UIA_ValueValuePropertyId,
                            VARIANT::from(a.as_str()),
                            VARIANT::from(b.as_str()),
                        )),
                        (Kind::Toggle(a), Kind::Toggle(b)) if a != b => property_changes.push((
                            index,
                            UIA_ToggleToggleStatePropertyId,
                            VARIANT::from(i32::from(*a)),
                            VARIANT::from(i32::from(*b)),
                        )),
                        _ => (),
                    }
                }
            }
            focus_id = app.focus_index.and_then(|i| nodes.get(i)).map(|n| n.id);
            focus_changed = m.focus != focus_id;
            m.nodes = nodes;
            m.focus = focus_id;
            m.active = true;
            epoch = m.epoch;
        }
        unsafe {
            if UiaClientsAreListening().as_bool() {
                for (index, property, old, new) in property_changes {
                    let p = self.provider(Some(index), epoch);
                    let _ = UiaRaiseAutomationPropertyChangedEvent(&p, property, &old, &new);
                }
                if changed {
                    let root = self.provider(None, 0);
                    let _ = UiaRaiseStructureChangedEvent(
                        &root,
                        StructureChangeType_ChildrenInvalidated,
                        std::ptr::null_mut(),
                        0,
                    );
                }
                if focus_changed {
                    if let Some(index) = focus_id {
                        let p = self.provider(Some(index), epoch);
                        let _ = UiaRaiseAutomationEvent(&p, UIA_AutomationFocusChangedEventId);
                    }
                }
            }
        }
    }
    pub fn take(&self, app: &App) -> Option<(usize, Action, Command)> {
        let mut m = self.0.lock().unwrap_or_else(|e| e.into_inner());
        while let Some(request) = m.pending.pop_front() {
            if m.alive && m.active && request.epoch == m.epoch {
                if let Some(position) = m.nodes.iter().position(|n| n.id == request.index) {
                    let node = &m.nodes[position];
                    if let Some(hit) = app.hits.get(position) {
                        if node.key == format!("{:?}", hit.action) {
                            return Some((position, hit.action.clone(), request.command));
                        }
                    }
                }
            }
        }
        None
    }
    pub fn unregister(&self, hwnd: HWND) {
        {
            let mut m = self.0.lock().unwrap_or_else(|e| e.into_inner());
            m.alive = false;
            m.active = false;
            m.pending.clear();
            m.nodes.clear();
        }
        let root = ROOTS.with(|r| r.borrow_mut().remove(&(hwnd.0 as isize)));
        if let Some(root) = root {
            unsafe {
                let _ = UiaDisconnectProvider(&root);
            }
        }
    }
}
pub unsafe fn get_object(hwnd: HWND, w: WPARAM, l: LPARAM) -> Option<LRESULT> {
    if l.0 as i32 != UiaRootObjectId {
        return None;
    }
    let root = ROOTS.with(|r| r.borrow().get(&(hwnd.0 as isize)).cloned());
    root.map(|r| UiaReturnRawElementProvider(hwnd, w, l, &r))
}
fn field_value(app: &App, field: Field) -> String {
    match field {
        Field::Search => app.search.clone(),
        Field::HelpSearch => app.help_search.clone(),
        Field::IconSearch => app.icon_search.clone(),
        Field::Naming => app.settings.naming.clone(),
        Field::ProjectName => app.settings.project_name.clone(),
        Field::ExportName => app.export_name.clone(),
        _ => app.text_buffer.clone(),
    }
}
fn toggle_value(a: &App, t: Toggle) -> bool {
    let s = &a.settings;
    match t {
        Toggle::AudioLimiter => s.audio_limiter,
        Toggle::AudioSources => s.audio_sources,
        Toggle::AutoFallback => s.auto_fallback,
        Toggle::Microphone => s.microphone,
        Toggle::SystemAudio => s.system_audio,
        Toggle::Webcam => s.webcam,
        Toggle::Cursor => s.cursor,
        Toggle::Clicks => s.click_effects,
        Toggle::Recovery => s.recovery,
        Toggle::Notifications => s.notifications,
        Toggle::Tray => s.tray,
        Toggle::Startup => s.startup,
        Toggle::GpuResize => s.gpu_resize,
        Toggle::Animated => s.appearance.animated,
        Toggle::MinimizeOnRecord => s.appearance.minimize_on_record,
        Toggle::ExcludeSelf => s.appearance.exclude_self,
        Toggle::ScreenshotClipboard => s.screenshot_clipboard,
        Toggle::ScreenshotCursor => s.screenshot_cursor,
        Toggle::WebcamMirror => s.webcam_mirror,
        Toggle::VoiceFilter => s.voice_filter,
    }
}
fn toggle_name(t: Toggle) -> &'static str {
    use crate::i18n::tr;
    match t {
        Toggle::Microphone => tr("Microfone"),
        Toggle::SystemAudio => tr("Áudio do sistema"),
        Toggle::Webcam => tr("Webcam"),
        Toggle::Cursor | Toggle::ScreenshotCursor => tr("Cursor"),
        Toggle::Recovery => tr("Recuperação"),
        Toggle::Notifications => tr("Notificações"),
        Toggle::Tray => tr("Minimizar para a bandeja"),
        Toggle::Startup => tr("Iniciar com o Windows"),
        Toggle::Animated => tr("Animações"),
        Toggle::ScreenshotClipboard => tr("Copiar imagem para colar"),
        Toggle::AudioLimiter => tr("Limitar picos de áudio"),
        Toggle::AudioSources => tr("Salvar fontes de áudio"),
        Toggle::AutoFallback => tr("Compatibilidade automática"),
        Toggle::Clicks => tr("Destacar cliques"),
        Toggle::GpuResize => tr("Redimensionamento pela GPU"),
        Toggle::MinimizeOnRecord => tr("Minimizar ao gravar"),
        Toggle::ExcludeSelf => tr("Ocultar o aplicativo da captura"),
        Toggle::WebcamMirror => tr("Espelhar imagem"),
        Toggle::VoiceFilter => tr("Melhorar voz"),
    }
}
fn fallback_name(a: &Action) -> &'static str {
    use crate::i18n::tr;
    match a {
        Action::Minimize => tr("Minimizar"),
        Action::Maximize => tr("Maximizar"),
        Action::Close | Action::CloseModal => tr("Fechar"),
        Action::Pin => tr("Sempre no topo"),
        Action::Pause => tr("Pausar"),
        Action::Stop => tr("Parar"),
        Action::Preview => tr("Prévia ao vivo"),
        Action::Screenshot => tr("Captura de tela"),
        _ => tr("Controle"),
    }
}

#[implement(
    IRawElementProviderSimple,
    IRawElementProviderFragment,
    IRawElementProviderFragmentRoot,
    IInvokeProvider,
    IValueProvider,
    IToggleProvider,
    IRangeValueProvider
)]
struct Provider {
    bridge: Bridge,
    index: Option<usize>,
    epoch: i32,
}
impl Provider {
    fn node(&self) -> Result<Node> {
        let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        // Readers may use the last complete snapshot while an asynchronous
        // action runs. Only new actions are blocked during a native modal loop.
        if !m.alive || self.epoch != m.epoch {
            return Err(unavailable());
        }
        self.index
            .and_then(|id| m.nodes.iter().find(|n| n.id == id))
            .cloned()
            .ok_or_else(unavailable)
    }
    fn root(&self) -> Result<()> {
        if self
            .bridge
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .alive
        {
            Ok(())
        } else {
            Err(unavailable())
        }
    }
    fn queue(&self, command: Command) -> Result<()> {
        self.node()?;
        let mut m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        if !m.active
            || self.epoch != m.epoch
            || m.pending.len() >= 32
            || !m.nodes.iter().any(|n| Some(n.id) == self.index)
        {
            return Err(HRESULT(UIA_E_INVALIDOPERATION as i32).into());
        }
        m.pending.push_back(Request {
            epoch: self.epoch,
            index: self.index.ok_or_else(unavailable)?,
            command,
        });
        unsafe {
            PostMessageW(
                Some(HWND(m.hwnd as *mut _)),
                ACTION_MESSAGE,
                WPARAM(0),
                LPARAM(0),
            )?;
        }
        Ok(())
    }
    fn fragment(&self, index: Option<usize>, epoch: i32) -> Result<IRawElementProviderFragment> {
        self.bridge.provider(index, epoch).cast()
    }
}
impl IRawElementProviderSimple_Impl for Provider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider | ProviderOptions_UseComThreading)
    }
    fn GetPatternProvider(&self, id: UIA_PATTERN_ID) -> Result<IUnknown> {
        if self.index.is_none() {
            self.root()?;
            return no_element();
        }
        match self.node()?.kind {
            Kind::Button if id == UIA_InvokePatternId => {
                Ok(self.to_interface::<IInvokeProvider>().into())
            }
            Kind::Edit(_) if id == UIA_ValuePatternId => {
                Ok(self.to_interface::<IValueProvider>().into())
            }
            Kind::Toggle(_) if id == UIA_TogglePatternId => {
                Ok(self.to_interface::<IToggleProvider>().into())
            }
            Kind::Range(..) if id == UIA_RangeValuePatternId => {
                Ok(self.to_interface::<IRangeValueProvider>().into())
            }
            _ => no_element(),
        }
    }
    fn GetPropertyValue(&self, id: UIA_PROPERTY_ID) -> Result<VARIANT> {
        self.root()?;
        let node = if self.index.is_some() {
            Some(self.node()?)
        } else {
            None
        };
        let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        Ok(match id {
            UIA_NamePropertyId => VARIANT::from(
                node.as_ref()
                    .map(|n| n.name.as_str())
                    .unwrap_or("WyzarCapture"),
            ),
            UIA_AutomationIdPropertyId => VARIANT::from(
                node.as_ref()
                    .map(|n| n.key.as_str())
                    .unwrap_or("WyzarCapture.Root"),
            ),
            UIA_FrameworkIdPropertyId => VARIANT::from("WyzarCapture.Win32"),
            UIA_ControlTypePropertyId => VARIANT::from(match node.as_ref().map(|n| &n.kind) {
                None => UIA_WindowControlTypeId.0,
                Some(Kind::Edit(_)) => UIA_EditControlTypeId.0,
                Some(Kind::Toggle(_)) => UIA_CheckBoxControlTypeId.0,
                Some(Kind::Range(..)) => UIA_SliderControlTypeId.0,
                _ => UIA_ButtonControlTypeId.0,
            }),
            UIA_IsControlElementPropertyId | UIA_IsContentElementPropertyId => VARIANT::from(true),
            UIA_IsEnabledPropertyId => VARIANT::from(m.active),
            UIA_IsKeyboardFocusablePropertyId => VARIANT::from(self.index.is_some()),
            UIA_HasKeyboardFocusPropertyId => {
                VARIANT::from(self.index.is_some() && self.index == m.focus)
            }
            UIA_IsOffscreenPropertyId => VARIANT::from(unsafe {
                !IsWindowVisible(HWND(m.hwnd as *mut _)).as_bool()
                    || IsIconic(HWND(m.hwnd as *mut _)).as_bool()
            }),
            UIA_NativeWindowHandlePropertyId if self.index.is_none() => {
                VARIANT::from(m.hwnd as i32)
            }
            _ => VARIANT::default(),
        })
    }
    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        self.root()?;
        if self.index.is_some() {
            no_element()
        } else {
            let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
            unsafe { UiaHostProviderFromHwnd(HWND(m.hwnd as *mut _)) }
        }
    }
}
impl IRawElementProviderFragment_Impl for Provider_Impl {
    fn Navigate(&self, d: NavigateDirection) -> Result<IRawElementProviderFragment> {
        self.root()?;
        if self.index.is_some() {
            self.node()?;
        }
        let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        let position = self
            .index
            .and_then(|id| m.nodes.iter().position(|n| n.id == id));
        let index = match (position, d) {
            (None, NavigateDirection_FirstChild) if self.index.is_none() && !m.nodes.is_empty() => {
                Some(m.nodes[0].id)
            }
            (None, NavigateDirection_LastChild) if self.index.is_none() && !m.nodes.is_empty() => {
                Some(m.nodes.last().unwrap().id)
            }
            (Some(_), NavigateDirection_Parent) => return self.fragment(None, 0),
            (Some(i), NavigateDirection_NextSibling) if i + 1 < m.nodes.len() => {
                Some(m.nodes[i + 1].id)
            }
            (Some(i), NavigateDirection_PreviousSibling) if i > 0 => Some(m.nodes[i - 1].id),
            _ => return no_element(),
        };
        self.fragment(index, m.epoch)
    }
    fn GetRuntimeId(&self) -> Result<*mut SAFEARRAY> {
        if self.index.is_none() {
            return Ok(std::ptr::null_mut());
        }
        self.node()?;
        unsafe {
            let array = SafeArrayCreateVector(VT_I4, 0, 3);
            if array.is_null() {
                return Err(E_OUTOFMEMORY.into());
            }
            for (i, value) in [
                UiaAppendRuntimeId as i32,
                self.epoch,
                self.index.unwrap() as i32 + 1,
            ]
            .iter()
            .enumerate()
            {
                if let Err(e) =
                    SafeArrayPutElement(array, &(i as i32), (value as *const i32).cast())
                {
                    let _ = SafeArrayDestroy(array);
                    return Err(e);
                }
            }
            Ok(array)
        }
    }
    fn BoundingRectangle(&self) -> Result<UiaRect> {
        if self.index.is_some() {
            return Ok(self.node()?.rect);
        }
        self.root()?;
        let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        let mut r = RECT::default();
        unsafe {
            GetWindowRect(HWND(m.hwnd as *mut _), &mut r)?;
        }
        Ok(UiaRect {
            left: r.left as f64,
            top: r.top as f64,
            width: (r.right - r.left) as f64,
            height: (r.bottom - r.top) as f64,
        })
    }
    fn GetEmbeddedFragmentRoots(&self) -> Result<*mut SAFEARRAY> {
        Ok(std::ptr::null_mut())
    }
    fn SetFocus(&self) -> Result<()> {
        if self.index.is_none() {
            self.root()
        } else {
            self.queue(Command::Focus)
        }
    }
    fn FragmentRoot(&self) -> Result<IRawElementProviderFragmentRoot> {
        self.root()?;
        self.bridge.provider(None, 0).cast()
    }
}
impl IRawElementProviderFragmentRoot_Impl for Provider_Impl {
    fn ElementProviderFromPoint(&self, x: f64, y: f64) -> Result<IRawElementProviderFragment> {
        self.root()?;
        let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        let found = m
            .nodes
            .iter()
            .enumerate()
            .rev()
            .find(|(_, n)| {
                x >= n.rect.left
                    && y >= n.rect.top
                    && x < n.rect.left + n.rect.width
                    && y < n.rect.top + n.rect.height
            })
            .map(|(_, n)| n.id);
        self.fragment(found, m.epoch)
    }
    fn GetFocus(&self) -> Result<IRawElementProviderFragment> {
        self.root()?;
        let m = self.bridge.0.lock().unwrap_or_else(|e| e.into_inner());
        self.fragment(m.focus, m.epoch)
    }
}
impl IInvokeProvider_Impl for Provider_Impl {
    fn Invoke(&self) -> Result<()> {
        if matches!(self.node()?.kind, Kind::Button) {
            self.queue(Command::Invoke)
        } else {
            unsupported()
        }
    }
}
impl IToggleProvider_Impl for Provider_Impl {
    fn Toggle(&self) -> Result<()> {
        if matches!(self.node()?.kind, Kind::Toggle(_)) {
            self.queue(Command::Invoke)
        } else {
            unsupported()
        }
    }
    fn ToggleState(&self) -> Result<ToggleState> {
        if let Kind::Toggle(on) = self.node()?.kind {
            Ok(if on { ToggleState_On } else { ToggleState_Off })
        } else {
            unsupported()
        }
    }
}
impl IValueProvider_Impl for Provider_Impl {
    fn SetValue(&self, text: &PCWSTR) -> Result<()> {
        if !matches!(self.node()?.kind, Kind::Edit(_)) {
            return unsupported();
        }
        let text = unsafe { text.to_string()? };
        if text.len() > 500 || text.chars().any(char::is_control) {
            return Err(E_INVALIDARG.into());
        }
        self.queue(Command::Text(text))
    }
    fn Value(&self) -> Result<BSTR> {
        if let Kind::Edit(text) = self.node()?.kind {
            Ok(BSTR::from(text))
        } else {
            unsupported()
        }
    }
    fn IsReadOnly(&self) -> Result<BOOL> {
        self.node()?;
        Ok(BOOL(0))
    }
}
impl IRangeValueProvider_Impl for Provider_Impl {
    fn SetValue(&self, value: f64) -> Result<()> {
        if let Kind::Range(_, min, max) = self.node()?.kind {
            if !value.is_finite() || value < min || value > max {
                return Err(E_INVALIDARG.into());
            }
            self.queue(Command::Number(value))
        } else {
            unsupported()
        }
    }
    fn Value(&self) -> Result<f64> {
        if let Kind::Range(value, ..) = self.node()?.kind {
            Ok(value)
        } else {
            unsupported()
        }
    }
    fn IsReadOnly(&self) -> Result<BOOL> {
        self.node()?;
        Ok(BOOL(0))
    }
    fn Maximum(&self) -> Result<f64> {
        if let Kind::Range(_, _, max) = self.node()?.kind {
            Ok(max)
        } else {
            unsupported()
        }
    }
    fn Minimum(&self) -> Result<f64> {
        if let Kind::Range(_, min, _) = self.node()?.kind {
            Ok(min)
        } else {
            unsupported()
        }
    }
    fn LargeChange(&self) -> Result<f64> {
        Ok(0.25)
    }
    fn SmallChange(&self) -> Result<f64> {
        Ok(0.05)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(key: &str) -> Node {
        Node {
            id: 0,
            key: key.into(),
            name: key.into(),
            rect: UiaRect::default(),
            kind: Kind::Button,
        }
    }
    #[test]
    fn filtering_items_keeps_the_search_control_identity_and_does_not_reuse_removed_ids() {
        let mut next = 1;
        let mut initial = vec![node("Search"), node("ItemA"), node("ItemB")];
        assign_ids(&mut initial, &[], true, &mut next);
        let mut filtered = vec![node("Search"), node("ItemB")];
        assign_ids(&mut filtered, &initial, false, &mut next);
        assert_eq!(initial[0].id, filtered[0].id);
        assert_eq!(initial[2].id, filtered[1].id);
        let mut added = vec![node("Search"), node("NewItem")];
        assign_ids(&mut added, &filtered, false, &mut next);
        assert_ne!(added[1].id, initial[1].id);
    }
    #[test]
    fn changing_pages_invalidates_identical_action_names() {
        let mut next = 1;
        let mut old = vec![node("Close")];
        assign_ids(&mut old, &[], true, &mut next);
        let mut new = vec![node("Close")];
        assign_ids(&mut new, &old, true, &mut next);
        assert_ne!(old[0].id, new[0].id);
    }
    #[test]
    fn asynchronous_actions_keep_the_last_readable_snapshot() {
        let bridge = Bridge::new();
        {
            let mut model = bridge.0.lock().unwrap();
            let mut n = node("Search");
            n.id = 7;
            model.nodes.push(n);
            model.active = true;
        }
        let provider = Provider {
            bridge: bridge.clone(),
            index: Some(7),
            epoch: 1,
        };
        bridge.suspend();
        assert!(provider.node().is_ok());
        assert!(provider.queue(Command::Focus).is_err());
        bridge.0.lock().unwrap().epoch = 2;
        assert!(provider.node().is_err());
    }
}
