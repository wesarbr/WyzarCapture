# Installation and updates

**Version:** 0.6.1 Beta for Windows x64. Get the official `WyzarCaptureSetup.exe` from [Releases](https://github.com/wesarbr/WyzarCapture/releases). Only the Setup executable is required. See [beta status](BETA_STATUS.md) for current availability and testing scope.

## Before installing

Use Windows x64 with the required Windows media components, graphics drivers and an interactive desktop session. The API baseline is Windows 10 version 2004; local validation currently covers Windows 11 Pro Insider Preview. The reported Windows 10 recording/camera issue still needs a retest on the affected notebook.

The candidate has no production Authenticode signature. Obtain official installers only from the project Releases page. If Windows or your security software blocks or detects the file, stop and report the detection through the documented channel. Keep protection enabled.

## Walk through the native Setup

1. Open the downloaded Setup. The payload is embedded; no runtime download is performed.
2. Use **Language** in the header to select Português (Brasil), English, Español, 简体中文 or 日本語. **License** opens the complete own terms in that language.
3. Continue to Destination. Choose the application folder separately from the media root. The beta installs for the current user.
4. Choose a media root in an available location that you can write to. Setup creates and checks the recording, audio, screenshot, project and export directories. You can use the current Windows Videos location or choose another folder with the native picker. If the automatic destination is unavailable, Setup presents a writable local alternative; review it and start installation again.
5. Review shortcut/startup choices, install and open the application.

Do not copy a previously installed data directory from another computer as a substitute for installation. Old paths or device identifiers may refer to unavailable resources.

## First launch and later changes

The installed executable applies the Setup media/language request once during normal startup. A new profile is proposed from the detected monitor and available hardware. Existing preferences remain in effect unless an explicit new Setup request is applied. Unavailable automatic defaults can receive a writable local alternative during first use. Explicit requests and existing preferences are preserved if validation fails, with a folder correction offered.

Change language in **Settings › General › Language**. Change media destinations in **Settings › Storage** using native folder selection and write validation. **Your folders** provides open/change actions for videos, audio, screenshots, cache, exports and projects. A shared media root prepares the media destinations while cache stays independent. Changing a destination does not move old files. Your later language choice survives restart and loading recording profiles.

Check picture and sound with a short recording before a longer session. [Quick start](QUICK_START.md).

## Update, repair and uninstall

Close recording and export sessions before updating. Use an official reviewed installer; review its update/repair mode and destination. Personal media and projects are preserved. Modified legacy documents can be retained rather than overwritten or removed silently.

Uninstall through Windows or the installed uninstaller. The normal removal preserves personal media and projects. Optional cleanup choices for preferences, cache and logs are separate; inspect them before confirming. Only runtime files belong in the installation: the application, uninstaller and installation receipt. Development instructions, sources and audit reports are not installed.

[Troubleshooting](TROUBLESHOOTING.md) · [Languages](LANGUAGES.md) · [Back to project](../README.md)
