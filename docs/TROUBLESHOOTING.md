# Troubleshooting

## "An installation already exists at ..." in Setup

Setup 0.6.3 and earlier refused a different destination while WyzarCapture was registered in another folder, for example `AppData\Local\Programs\WyzarCapture`, even when the page said "New installation". Setup 0.7.0 shows the current folder and version on the Destination page. Choose **Use current folder** to update in place, or keep the new folder and select **Install** to move the program. The new folder is installed and registered first; then only the old program files are removed. Videos, projects, preferences and files you added to the old folder stay where they are. If WyzarCapture is open from the old folder, Setup asks to close it cleanly first.

If the previously registered folder was deleted, Setup 0.7.0 treats the entry as stale and installs normally. With an older Setup, install into the folder shown in the message or uninstall it from Windows Settings › Apps first.

## Windows media folders or another drive are unavailable

Since 0.6.3, the automatic AppData media fallback reported in 0.6.1 is corrected. It resolves Videos, Pictures, Music and Documents through Windows and keeps media separate from installation. In Setup, selecting a drive uses its WyzarCapture subfolder. In the application, use Your folders to inspect, open or change the destination, or Restore Windows folders.

A disconnected drive, inaccessible redirected folder or Windows protection can still prevent writing. Keep protection enabled and choose an available writable destination. OS error 2 indicates a path/file was not found; it does not by itself prove that an access permission was denied. No software can promise access to every protected folder.

The affected Windows 10 notebook still needs a real retest with the exact new installer. Existing media is preserved when destinations change. The old automatic fallback receives a preference backup before migration; other custom choices remain intact.

## A theme or menu seems to need two clicks

The corrected version clears previous focus on theme/category changes and rebuilds hit areas before the next input. Clicking outside an open selector also activates the visible control in that click. If a different control still fails, report the version, interface mode, theme, Windows scaling and exact steps without personal paths.


## Recording cannot create a destination

Open **Your folders** or **Settings › Storage**, use Restore Windows folders or choose an available folder with the native picker, and retry a short recording. Windows account permissions and security protections remain in effect. First launch prepares the Windows media destinations; Setup validates an explicitly selected custom media root. Redirected or disconnected folders, permission restrictions and stale paths copied from another computer can still need correction. Free disk space alone does not prove a destination is available.

If it fails, use the operation's diagnostic copy action. Record the OS error number and which action failed. Remove personal paths before sharing screenshots or manually copied error text. A failed new folder request should not replace saved working preferences.

## Not enough space to finalize a recording

Since 0.6.3, recording keeps a safety reserve and, with recovery enabled, room for a second MP4 copy during assembly. Free space can therefore be required even when there is room for one more frame. Choose another destination or free space. Preserve the recovery segments if final assembly reports insufficient space; do not assume a partial export is complete.

## A monitor or audio device is missing

Check that the device is available in Windows and connected in the current desktop session. Reopen the appropriate selector when idle. Device/display changes and the start of recording refresh inventory and reconcile stale selections. A failure in one category should leave other categories available.

Check microphone permissions and the selected input/output endpoints. System audio follows the chosen playback endpoint; an unused endpoint can produce silence. Reconnect devices while idle and retry a short test.

## Camera cannot start

Confirm Windows camera access for desktop applications, check the driver and close other applications that may be holding the camera. Select the listed camera explicitly and retry while idle. The candidate tries bounded native formats and advanced/legacy processing paths, validates buffer dimensions/stride and preserves source ownership while retrying.

If the error persists, disable the camera temporarily to test screen recording independently. Report the camera model, Windows build, selected mode, stage and numeric HRESULT. The Windows 10 notebook's physical camera remains a required retest; these changes are not a camera compatibility certification.

## Screenshot or shortcut does not behave as expected

Open Screenshot and confirm the selected source and target, delay, cursor and image-copy options. **Capture now** uses these settings and saves a PNG in the screenshot destination. Reopen the folder from the saved notice or Your folders. Screenshot dimensions follow the source, rather than the video quality preset.

In Settings › Shortcuts, press the intended key combination and check for duplicate or system-conflict messages. Choose another combination if Windows or another application has reserved it. Restore defaults if needed. On systems where Windows handles PrintScreen itself, check the Windows screenshot-key setting or use a different WyzarCapture shortcut.

## Recording is slow or drops frames

Reduce resolution/frame rate, close competing workloads, check free space and use a suitable local destination. A selected quality profile and format negotiation do not prove sustained capture rate. Hardware encoding availability depends on drivers and media transforms. Verify actual exported playback.

## A window or game is black

Try a normal visible window or windowed/borderless game. Protected content and some application/game rendering paths can prevent Windows Graphics Capture. An exclusive game capture hook is not provided.

## Language, fonts or layout

Use **Settings › General › Language**, or the installer's header selector. Language names remain recognizable as autonyms. Chinese is Simplified. Windows dialogs/system messages can remain in the Windows language; file and device names are data and are preserved. Report missing glyphs or clipping with your selected language and scaling. See the [language guide](LANGUAGES.md).

## An interrupted recording or missing project source

Check Recovery for completed segments. Preserve originals if an operation fails. Recovery may lose the last open segment and is not guaranteed after disk/power failure. Projects reference source media; restore the source location if it has moved or disappeared.

## Security software blocks Setup

Keep protection enabled. Do not add an exclusion or restore a quarantined file just to make installation proceed. Preserve the detection name and official asset/version information; follow the [security policy](../SECURITY.md). The candidate is unsigned.

[Installation](INSTALLATION.md) · [Quick start](QUICK_START.md) · [Back to project](../README.md)
