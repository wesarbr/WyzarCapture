# Beta status and tested scope

**Current application and documentation: 0.6.1 Beta, Windows x64.** The official distribution channel is [GitHub Releases](https://github.com/wesarbr/WyzarCapture/releases), with `WyzarCaptureSetup.exe` as the installer asset. The installer upload is not yet completed; documentation preparation does not establish download availability. This status will be updated only after the Release and its asset are verified.

## What changed in 0.6.1

- Configurable screenshots: monitor, window, region or recording source, optional delay/cursor/image copy and original-dimension PNGs.
- Key capture for customizable shortcuts, duplicate checks, conflict messages and default restoration.
- Your folders panel, shared media-root preparation and explicit writable local recovery while preserving existing choices and media.
- Detected monitor preview at normal startup and compact controls; expanded camera preview when a device is available.
- Editor fixes for unsaved changes after undo/redo and cumulative timing across short cuts.
- New controls, help and Setup messages in all five built-in languages.

[Complete release notes](releases/0.6.1.md).

## Evidence and its boundaries

The exact application and installer completed the local release pipeline on Windows 11 Pro Insider Preview. Checks included offline builds, unit tests, real current-user installation and normal startup, writable destinations, short screen/audio captures, decoding, editing/export, recovery, language persistence, native rendering and normal local security scans. Internal reports and artifact integrity records stay outside this documentation portal.

The editor checks exercised import, playback, seeking, splitting, trimming, removal, undo/redo, speed/volume, basic effects, saving/reopening, export progress and decoding. A real export of forty 50 ms cuts checked the timing regression. These checks establish their tested cases, not universal correctness for every media file or editing combination.

Renderer scale matrices are not physical DPI tests. Camera layouts used explicitly synthetic content; source-reader retries are not physical camera tests. Short recordings are not hours-long endurance certification. Physical key entry, folder-picker interaction and pasting an image into another application still need interactive retests. Production Authenticode signing is absent.

## Windows 10 notebook and camera retest

A Windows 10 notebook previously failed when preparing a recording destination and enabling its camera. The affected hardware is not present in the local validation environment. Retest installation, chosen folders, monitor detection, a short recording, playback, camera activation, expanded preview, screenshots, custom keys, restart and language persistence on that notebook before calling the reported issue resolved.

## Known feature limits

- No certified exclusive game hook or universal protected-content/anti-cheat capture.
- Webcam composited into video; no independent camera-video track.
- Independent WAV mixing exists; video cuts do not automatically ripple/re-align those tracks.
- Basic effects have limited parameters; full keyframe editing and complete accessibility remain future work.
- Recovery can preserve completed segments, but may lose the open segment and cannot guarantee recovery after power or disk failure.
- Other GPU vendors, stable Windows builds, physical cameras/scales and long sessions need broader validation.
- Production installation is current-user only; all-users installation is disabled.

[Troubleshooting](TROUBLESHOOTING.md) · [Back to project](../README.md)
