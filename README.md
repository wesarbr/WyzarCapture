<p align="center"><img src="assets/github/logo.png" width="390" alt="WyzarCapture"></p>
<p align="center"><strong>Capture your screen. Find your recordings. Keep creating.</strong></p>
<p align="center">Windows x64 · 0.6.0 Beta candidate · Native desktop application · Free to use</p>
<p align="center"><a href="docs/BETA_STATUS.md"><img src="assets/github/download.svg" width="360" alt="Beta status — installer release pending"></a></p>
<p align="center"><a href="docs/INSTALLATION.md">Installation</a> · <a href="docs/QUICK_START.md">Quick start</a> · <a href="docs/LANGUAGES.md">Languages</a> · <a href="docs/GALLERY.md">Gallery</a> · <a href="docs/FAQ.md">FAQ</a></p>

![WyzarCapture native screen recording, editing and compact controls](assets/github/hero.png)

> **Private beta preview — installer release pending.** This repository presents WyzarCapture and its documentation for the owner's review. A GitHub release and installer download are not available yet. The owner controls when this repository becomes public. This Windows-only beta candidate is unsigned and still requires a retest on the Windows 10 notebook that reported recording and camera errors. See [beta status](docs/BETA_STATUS.md).

## Your recording workflow, on your computer

WyzarCapture is a native Windows application for screen and audio recording, a local media library, basic editing and MP4 export. Choose what to capture, record it, find the result and prepare a version to share in one application.

Your files stay in folders you choose. Recording, previews, editing and export run locally. No online account is required, and this version implements no telemetry or automatic media uploads. The self-contained installer uses the Windows media components already on your computer.

| Step | What you can do |
| --- | --- |
| **Capture** | Select a monitor, window or region; choose resolution and frame rate; pause, resume and save PNG screenshots. |
| **Record sound** | Combine microphone and system sound, record audio alone as WAV, or record the screen with separate WAV companion files. |
| **Organize** | Browse thumbnails, search names and tags, use favorites, switch grid/list views and inspect file information. |
| **Edit** | Trim, split and remove clips; adjust speed and volume; add basic text, arrows, central blur, crop and fade effects. |
| **Mix** | Work with associated or imported WAV tracks using mute, solo, gain, offset, fades and nondestructive cuts. |
| **Export** | Create a new H.264/AAC MP4 with a chosen name, destination, resolution, frame rate and bitrate. |
| **Personalize** | Use eight themes or a custom palette, change shortcuts, save local recording profiles and choose your language. |

### A window that fits the task

The standard layout brings together preview, sources and devices. Focus, Compact and Compact focus layouts adapt the controls to your workspace. Compact mode provides start, pause and stop, recording choices, a destination control and an always-on-top option.

![Compact recording controls in the native application](assets/github/compact.png)

### Five languages, built in

Choose **English, Spanish, Simplified Chinese, Japanese or Brazilian Portuguese** in the installer's header or in **Settings › General › Language** after installation. The application changes immediately and saves your preference. Translations are embedded; no language packs are downloaded.

Your file names, folders, device names and content are preserved. Windows-provided dialogs and system messages follow the Windows language, and original third-party legal notices retain their original wording. [Language guide](docs/LANGUAGES.md).

![Language selector in WyzarCapture](assets/github/languages.png)

### Find the recording and continue in the editor

The Library brings together thumbnails, metadata, favorites and search. Projects use `.wyzproj` files that reference original media. Editing and exporting create another video; keep the original media available to reopen your project.

![Native editor with synthetic color-bar demonstration media](assets/github/editor.png)

Effects have basic, limited parameters. Video cuts do not automatically realign independent WAV tracks. Webcam video is composited into the recording rather than saved as a separate editable camera track. Review the exported file before sharing it.

<details>
<summary><strong>Explore the Library, themes and installer</strong></summary>

**Library — browse and manage your local files.**

![Library with synthetic demonstration media](assets/github/library.png)

**Appearance — eight presets and a custom palette.**

![Appearance settings](assets/github/appearance.png)

**Installer — language, application destination and media folder choices.**

![Native installer welcome page](assets/github/installer.png)

</details>

The images show the application's native renderer with isolated demonstration data and synthetic color bars. They are product illustrations, not performance benchmarks or evidence of physical camera testing. [Gallery context](docs/GALLERY.md).

## Install and make your first recording

Once the owner releases the reviewed candidate, the planned Windows asset is **WyzarCaptureSetup-0.6.0-Beta-Windows-x64.exe** in [GitHub Releases](https://github.com/wesarbr/WyzarCapture/releases). This is a planned asset name; no working download is claimed in this preview.

1. Download the official Setup when available and choose your language in its header.
2. Choose the application folder, a writable media folder and your shortcuts. Installation is for the current Windows user.
3. Open WyzarCapture, check the detected monitor and audio devices, and choose a source.
4. Make a short test, stop, then reopen it from the Library to check picture and sound.

The first launch proposes a profile using detected hardware and prepares your chosen folders for recordings, audio, screenshots, exports and projects. You can change destinations and recording settings later. Existing preferences are preserved.

Only the Setup executable is needed. You do not need Rust, developer tools or documentation files. After the initial download, installation and the described media functions use local resources. **Default shortcuts:** F8 starts, F9 pauses/resumes and F10 stops; change these in Settings › Shortcuts. F1 opens contextual help. [Full quick start](docs/QUICK_START.md).

## Requirements and beta boundaries

| Item | Requirement or current evidence |
| --- | --- |
| System | Windows x64; the APIs used require Windows 10 version 2004 or later. This is a technical baseline, not certification of every supported build. |
| Tested environment | Local Windows 11 Pro Insider Preview. The reported Windows 10 notebook remains a required retest. |
| Graphics | Direct3D 11 graphics and capture support in an interactive desktop session. |
| Media | Windows multimedia components and required codecs. Windows N systems may need their system media components configured. |
| Storage | Writable local or available redirected destinations, with room for recordings, temporary segments and exports. |
| Devices | Microphone/camera permissions, available drivers and devices for the optional functions you enable. |

Automatic detection and conservative defaults help prepare recording; they do not guarantee a sustained frame rate. Actual performance depends on the source, drivers, encoder and storage. Game mode captures a selected window with Windows Graphics Capture. Exclusive game hooks, protected-content capture and universal anti-cheat compatibility are not implemented or certified.

Interrupted sessions can sometimes be reconstructed from completed segments. The last open segment may be lost; recovery is not guaranteed after power loss or disk damage. Physical camera models, other GPU vendors, physical DPI settings and hours-long sessions need broader testing. [Detailed status](docs/BETA_STATUS.md) · [Troubleshooting](docs/TROUBLESHOOTING.md).

## Privacy, license and feedback

Your media stays under your control. Read the [privacy information](PRIVACY.md) for local data, retention and safe diagnostic sharing.

WyzarCapture is **freeware with proprietary code**. Personal, educational and professional use, including creating commercial content, is permitted under the [Freeware License](LICENSE.md). Source modification, redistribution and reverse engineering are restricted subject to mandatory legal exceptions. The license does not claim technical prevention of reverse engineering and does not take ownership of your recordings.

This repository is a distribution and documentation portal. It does not publish the proprietary source code, internal tests, audit records or signing material. [Change log](CHANGELOG.md) · [Security policy](SECURITY.md).

When the repository is available to you, use its [Issues](https://github.com/wesarbr/WyzarCapture/issues) for reproducible bugs and suggestions. Include your Windows build, application version, selected language, steps and whether the problem persists with a writable local folder. Remove personal paths, names and private media. Follow the security policy for confidential reports.
