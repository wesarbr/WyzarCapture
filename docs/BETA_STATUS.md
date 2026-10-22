# Beta status and tested scope

**0.6.0 is a private candidate under owner review. No GitHub release or working installer download has been published from this preparation.** The repository must remain private until the owner changes its visibility.

## What changed

- Five embedded interface languages, selectable in Setup and the application, with persistent preferences and complete own license texts.
- Initial folder preparation using the Windows Videos location or an explicit writable choice, with a user-local fallback and write checks.
- Separate application/media destination choices in Setup and a one-time handoff to the installed application.
- Monitor/device inventory and selection reconciliation, provisional hardware-based defaults, and stronger camera negotiation and RGB32 stride validation.
- Camera source ownership preserved while retrying Media Foundation reader configuration; errors identify the stage instead of silently discarding the cause.
- Text-height measurement for translated help and update history.

## Evidence and its boundaries

Builds and unit tests use the pinned local Rust/vendor sources offline. The release process checks real installation, normal installed-application startup, preferences, short screen/audio recording, media reopening/decoding, export, recovery, native renders and normal local security scans. Language checks exercise native renderers and application actions in five languages. Internal records remain private and are not shipped here.

These are bounded local checks on Windows 11 Pro Insider Preview. Renderer scale matrices are not physical DPI tests. Source-reader retry validation is not a physical camera test. Short recordings are not hours-long endurance certification. Successful scanning is an observation for the exact local artifact, not a universal security guarantee. Production Authenticode signing is absent.

## Required remote retest

A Windows 10 notebook previously failed when preparing a recording destination and enabling its camera. Folder and camera paths were corrected, but the affected hardware is not present in this validation environment. Retest installation, chosen folders, monitor detection, a short recording, playback, camera activation, restart and language persistence on that notebook before calling the report resolved.

## Known feature limits

- No certified exclusive game hook or universal protected-content/anti-cheat capture.
- Webcam composited into video; no independent camera-video track.
- Independent WAV mixing exists; video cuts do not automatically ripple/re-align those tracks.
- Basic effects have limited parameters; full keyframe editing and complete accessibility remain future work.
- Recovery can preserve completed segments, but may lose the open segment and cannot guarantee recovery after power or disk failure.
- Other GPU vendors, stable Windows builds, physical cameras/scales and long sessions need broader validation.
- Production installation is current-user only; all-users installation is disabled.

[Troubleshooting](TROUBLESHOOTING.md) · [Back to project](../README.md)
