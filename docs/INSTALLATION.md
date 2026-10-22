# Installation and updates

**Availability:** this private preview has no published installer download yet. The planned asset is `WyzarCaptureSetup-0.6.0-Beta-Windows-x64.exe` in the owner's [official Releases](https://github.com/wesarbr/WyzarCapture/releases).

## Before installing

Use Windows x64 with the required Windows media components, graphics drivers and an interactive desktop session. The API baseline is Windows 10 version 2004; local validation currently covers Windows 11 Pro Insider Preview. The reported Windows 10 recording/camera issue still needs a retest on the affected notebook.

The candidate has no production Authenticode signature. Obtain it only from the official channel when released. If Windows or your security software blocks or detects the file, stop and report the detection through the documented channel. Keep protection enabled.

## Walk through the native Setup

1. Open the downloaded Setup. The payload is embedded; no runtime download is performed.
2. Use **Language** in the header to select Português (Brasil), English, Español, 简体中文 or 日本語. **License** opens the complete own terms in that language.
3. Continue to Destination. Choose the application folder separately from the media root. The beta installs for the current user.
4. Choose a media root in an available location that you can write to. Setup creates and checks the recording, audio, screenshot, project and export directories. You can use the proposed Windows Videos location or choose another folder with the native picker.
5. Review shortcut/startup choices, install and open the application.

Do not copy a previously installed data directory from another computer as a substitute for installation. Old paths or device identifiers may refer to unavailable resources.

## First launch and later changes

The installed executable applies the Setup media/language request once during normal startup. A new profile is proposed from the detected monitor and available hardware. Existing preferences remain in effect unless an explicit new Setup request is applied. A failed destination check preserves the saved preferences and offers a folder correction.

Change language in **Settings › General › Language**. Change media destinations in **Settings › Storage** using native folder selection and write validation. Changing a destination does not move old files. Your later language choice survives restart and loading recording profiles.

Check picture and sound with a short recording before a longer session. [Quick start](QUICK_START.md).

## Update, repair and uninstall

Close recording and export sessions before updating. Use an official reviewed installer; review its update/repair mode and destination. Personal media and projects are preserved. Modified legacy documents can be retained rather than overwritten or removed silently.

Uninstall through Windows or the installed uninstaller. The normal removal preserves personal media and projects. Optional cleanup choices for preferences, cache and logs are separate; inspect them before confirming. Only runtime files belong in the installation: the application, uninstaller and installation receipt. Development instructions, sources and audit reports are not installed.

[Troubleshooting](TROUBLESHOOTING.md) · [Languages](LANGUAGES.md) · [Back to project](../README.md)
