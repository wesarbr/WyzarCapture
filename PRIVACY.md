# Privacy information

This page describes WyzarCapture 0.6.1 Beta and the prepared 0.6.3 Beta update.

## Processing on your computer

Capture, audio recording, thumbnails, editing, playback and export run locally. This version implements no required online account, telemetry, server-side metric collection, automatic recording upload or silent network updater. Initial GitHub downloads use internet. Windows and GitHub have their own service behavior and privacy terms.

## Local data

| Data | Purpose |
| --- | --- |
| Videos, WAV audio and PNG screenshots | Output in your configured destinations. |
| Projects and autosaves | Source references and editing decisions; may contain file names and paths. |
| Preferences and profiles | Language, folders, devices, shortcuts, appearance and recording configuration. |
| Library index and metadata | File locations, titles, tags, favorites and organization. |
| Recovery journals and segments | Attempted reconstruction of completed portions of interrupted sessions. |
| Cache and bounded local logs | Previews, normal operation and diagnostics. |

Normal installation stores preferences, index and logs in local Windows application data. Media, cache, project and export destinations are configurable. Language changes do not rename your files. Destination changes do not automatically move previous content.

## What you choose to capture

Your selected screen/window/region, microphone, system sound and camera determine saved content. Obtain permission where needed. Basic blur does not automatically identify sensitive information. Review exports before sharing.

Optional screenshot image-copy places the captured image on the Windows clipboard so you can paste it. Disable this option when you only want a file; other applications and Windows clipboard features can access clipboard content according to your system settings.

In 0.6.3, Windows UI Automation exposes names, values, focus and actions for supported interface controls to local accessibility clients. This does not upload control content to a server.

## Retention and removal

Normal updating/uninstalling preserves personal media and projects. Optional removal of preferences, cache and logs is separate. Modified legacy installation documents may be preserved. Library deletion asks for confirmation for recognized companion files before moving them to the Recycle Bin. Cleanup is not secure disk erasure.

## Sharing diagnostics

The application does not automatically send a report. Filing an Issue or sharing a screenshot/log is your action and follows GitHub visibility and terms. Copied diagnostics exclude private paths/free-form error text while retaining useful numeric codes; manually shared screenshots and files can still reveal information. Remove names, paths, credentials, conversations and third-party content. Prefer a small synthetic reproduction. Do not upload whole data folders or personal recordings.

[Security reporting](SECURITY.md) · [Back to project](README.md)
