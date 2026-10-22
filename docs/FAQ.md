# Frequently asked questions

## Is WyzarCapture free?

Yes. The own license permits personal, educational and professional use, including producing commercial content, without a license fee. It is proprietary freeware. Source modification, redistribution and reverse engineering are restricted subject to mandatory legal exceptions. Read the [license](../LICENSE.md).

## Does it work outside Windows?

This version is Windows x64 only. It uses native Windows capture, graphics, audio and media components. There is no macOS, Linux, browser or mobile build in this beta.

## Which languages are included?

Brazilian Portuguese, English, Spanish, Simplified Chinese and Japanese. Select a language in Setup or in Settings › General › Language. All translations are embedded. See [Languages](LANGUAGES.md).

## Do I need an account or internet?

No online account is required by the application. Downloading an official release uses internet. After download, Setup and the described recording/editing functions run locally using installed Windows components. This version implements no telemetry or automatic media upload.

## Does automatic setup guarantee recording on my PC?

No. Initial setup detects available hardware, proposes a profile and checks writable destinations. Permissions, drivers, codecs, disconnected destinations and device compatibility can still require changes. The reported Windows 10 notebook needs a new physical retest. Make a short test and inspect playback.

## Can I record games or a webcam?

Game mode uses a selected window through Windows Graphics Capture; exclusive hooks and universal anti-cheat/protected-content capture are not implemented. Webcam support has negotiation and buffer checks, but the affected physical notebook camera is not certified. Webcam video is composited into the recording.

## Are audio tracks fully independent in the editor?

Associated/imported WAV files have independent mix controls. Video cuts do not automatically realign those WAVs. Check offset, fades, preview and export manually. Independent camera video and full keyframe editing remain future work.

## Will an update or uninstall delete recordings?

The normal update/uninstall preserves personal media and projects. Optional preferences/cache/log cleanup is separate. Inspect all choices and retain backups. Changing a destination does not move existing files.

## Can I customize PrintScreen and recording keys?

Yes. Screenshot controls select the source, delay, cursor and optional image copy. In Settings › Shortcuts, click an action and press a key or modifier combination. Defaults can be restored. Windows reservations and conflicts still apply. See the [quick start](QUICK_START.md).

## Why are there no camera controls on my computer?

Camera controls appear when Windows enumerates a camera. When one is available, its expanded preview keeps the full image in proportion. Viewing it and including it in a recording are separate actions. Check Windows camera permissions if the expected device is missing.

## Can I recover everything after a crash?

Recovery uses completed segments and checkpoints. The final open segment may be lost, and recovery after power loss or disk damage is not guaranteed.

## Where do I get this version, and is it signed?

Use the project [Releases](https://github.com/wesarbr/WyzarCapture/releases) and select the official Windows x64 Setup. Version 0.6.1 Beta has no production Authenticode signature. Repository access follows the visibility settings chosen by the owner. See [beta status](BETA_STATUS.md) for availability and tested scope.

[Back to project](../README.md)
