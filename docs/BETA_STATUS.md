# Beta status and release checklist

**Prepared update: 0.7.0 Beta. Current download: [0.6.3 Beta](https://github.com/wesarbr/WyzarCapture/releases/tag/WyzarCapture).** The 0.7.0 installer has completed local validation but is not attached to a GitHub Release yet. Its new installer behavior and the interface shown in this portal are therefore a preview of that update. Repository visibility remains controlled by the owner. Executable integrity values are in [Checksums](CHECKSUMS.md).

| Version | Distribution state |
| --- | --- |
| 0.7.0 Beta | Documentation, product images and reference source prepared; installer upload pending. |
| 0.6.3 Beta | Setup is attached to the existing `WyzarCapture` Release. The service-reported asset size and digest match the local 0.6.3 Setup; this metadata check is not a fresh download comparison. |
| 0.6.1 Beta | Historical version; the current asset on the existing Release is 0.6.3. |

The old 0.5.1 detection applies to that withdrawn artifact; it does not describe the current executable. Its root cause remains undetermined. The current installer uses a conventional RCDATA resource and no payload overlay.

## What 0.7.0 corrects

The installer reported on a Windows 10 notebook showed "New installation" and then refused another folder because WyzarCapture was registered under `AppData\Local\Programs`. Setup 0.7.0 shows the registered folder and version, offers **Use current folder**, and moves the program after the new folder is installed and registered. Stale entries whose folder was deleted no longer block installation. Error messages have their own area. These cases were reproduced in an isolated current-user registry namespace, not on the reported notebook.

## Release checklist

| Review | Current evidence and remaining boundary |
| --- | --- |
| Defender on the exact EXEs | Final 0.7.0 application and Setup scanned normally without detection observed; hashes are bound to the local reports. This is not a universal security guarantee. |
| Self-contained Setup | Runtime-only RCDATA payload, no overlay, fixed file allow-list and integrity checks. |
| Installer behavior | 60 real current-user checks: install, update, repair, downgrade refusal, rollback, cancellation, concurrency, uninstall, moving an installation and stale entries. No remote shell, runtime download or service. |
| Hashes | Application and Setup SHA-256 values are on the dedicated checksum page. Only the official EXE is an installer download. |
| Credentials and private data | Documentation and the `source` folder are scanned for user paths, account names, tokens and private keys before publication. |
| Authenticode | Still unsigned: no production signing credential is configured. |
| Stable Windows builds | Local testing uses Windows 11 Pro Insider Preview. Windows 10 and stable Windows 11 still require their own retests. |
| UAC/Program Files/HKLM | All-users production installation is disabled. The beta installs for the current user only. |
| UI interaction | Controller/media tests and real UI Automation client actions passed. Physical mouse/keyboard/folder-picker tests remain separate and pending. |
| Longer capture | One ten-minute 720p/30 capture completed with CPU/RAM/disk sampling, reopening/decoding and owned-media cleanup. This does not certify hours-long sessions or every quality profile. |
| Compatible capture | The GDI fallback recorded, reopened and exported a short sample when forced on this computer. It is slower than Windows Graphics Capture and also sees overlapping windows. |
| Webcam and microphone | No camera was enumerated here. Native-mode selection, parallel reading and the shaped overlay are covered by code tests; the voice filter by synthetic signals. Physical camera and microphone retests remain required. |
| AMD/Intel/other PCs | Not certified by this local environment. |
| Physical DPI/multi-monitor | Renderer scales are covered; physical 125/150/200% DPI and multiple-monitor combinations remain untested. |
| Accessibility/UIA | Basic providers, names, bounds, navigation, focus, Invoke, Value, Toggle and RangeValue are implemented and tested. Complete TextPattern, screen-reader behavior and certification remain pending. |
| macOS/Linux | Not available. A port needs its own platform layers, test machines and signing. |

## 0.7.0 local validation

The exact application and Setup passed the local 33-report pipeline: 75 unit tests per binary, 60 real current-user installer checks (including moving an installation registered in another folder, refusing a move without confirmation and replacing a stale entry), capture, editing and export, recovery, 405 native renders and 65 checks across five languages, 50 usability renders and 8 Windows UI Automation client actions. A ten-minute 720p/30 recording (600.6 s) was reopened and decoded at the start, middle and end. With the new variable bitrate it used 164 MB of the disk, compared with 443 MB for the same profile and kind of desktop content in 0.6.3. These desktop samples are illustrative, not a controlled benchmark or a guaranteed size reduction. Generated test media was hashed and removed.

## Known feature limits

- Basic effects and independent WAV mixing remain; video cuts do not automatically realign independent WAV tracks.
- Camera video remains composited, with no separate editable camera-video track.
- Exclusive game hooks and universal protected-content/anti-cheat capture are not implemented or certified.
- Recovery may lose the last open segment. Full keyframes and complete accessibility remain future work.

[0.7.0 notes](releases/0.7.0.md) · [Installation](INSTALLATION.md) · [Troubleshooting](TROUBLESHOOTING.md) · [Back to project](../README.md)
