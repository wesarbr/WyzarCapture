# WyzarCapture application source

This folder contains the application source code of WyzarCapture 0.7.0 Beta for reference: the Rust modules of the recorder, editor, library, installer interface, Windows platform layer, embedded translations, license texts, icons and brand assets.

It is provided under the same [Freeware License](../LICENSE.md) as the application. The code remains proprietary: viewing it does not grant rights to modify, redistribute or publish derived versions beyond what the license allows.

Not included: internal test harnesses and their records, release and packaging tooling, third-party dependency sources, signing material and development notes. This folder is therefore not a complete build tree. Official installers are published only on the [Releases page](https://github.com/wesarbr/WyzarCapture/releases).

| Folder | Contents |
| --- | --- |
| `src/ui` | Application state, actions, views, theme and vector icons |
| `src/platform/windows` | Win32 window, Direct2D/DirectWrite renderer, screen capture, audio, camera, Media Foundation, files and installer effects |
| `src/capture`, `src/audio`, `src/encoder` | Recording session, audio processing and encoding configuration |
| `src/editor`, `src/library`, `src/recovery` | Editing, local library and interrupted-session recovery |
| `src/installer` | Installer model, transaction and interface |
| `src/i18n`, `localization/legal` | Embedded translations and license texts |
| `assets` | Brand, icons, theme tokens, application manifest and resource script |
