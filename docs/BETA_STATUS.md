# Beta status and release checklist

**Older download with reported folder issues: [0.6.1 Beta](https://github.com/wesarbr/WyzarCapture/releases/tag/WyzarCapture). Prepared update: 0.6.3 Beta.** Use the new installer for retesting once it is attached. The newer application and Setup completed local validation but are not attached to a new GitHub Release yet. Repository visibility remains controlled by the owner. Executable integrity values are in [Checksums](CHECKSUMS.md).

The old 0.5.1 detection applies to that withdrawn artifact; it does not describe the current executable. Its root cause remains undetermined. The current installer uses a conventional RCDATA resource and no payload overlay. This is an architectural change, not proof of the historical detection trigger.

## Corrected folder and selection behavior

Windows destinations are separated by file type and respect redirection; the automatic AppData media fallback is removed. The complete old fallback layout receives a preference backup and future-destination migration while media stays in place. A drive selection uses a product subfolder. Theme/menu selection and the first click outside selectors are checked across nine themes, ten categories and five languages. A native usability pass produced 50 renders; it is distinct from physical input on the affected notebook.

## Release checklist

| Review | Current evidence and remaining boundary |
| --- | --- |
| Defender on the exact EXEs | Final 0.6.3 application and Setup scanned normally without detection observed; hashes are bound to the local reports. This is not a universal security guarantee. |
| Self-contained Setup | Runtime-only RCDATA payload, no overlay, fixed file allow-list and integrity checks. Historical comparison is preserved; the withdrawn file is not restored or rebuilt. |
| Installer behavior | Current-user directories, registry and explicit shortcut/startup choices checked. Inspected production call sites do not invoke a remote shell, download runtime components or install a service. |
| Hashes | Application and Setup SHA-256 values are published on the dedicated checksum page at the owner’s request. Only the official EXE is an installer download. |
| Credentials and private data | Pattern checks covered all three reachable documentation commits and 32 unique text blobs before this update; no credential/private-path matches were found. Proprietary sources and private audits remain outside this portal. |
| Install/update/repair/rollback/uninstall | Real current-user tests passed with preservation cases for media, projects and modified legacy documents. |
| Authenticode | Still unsigned: no production signing credential is configured. |
| Stable Windows builds | Local testing uses Windows 11 Pro Insider Preview. Windows 10 and stable Windows 11 still require their own retests. |
| UAC/Program Files/HKLM | All-users production installation is disabled. The beta uses HKCU/current-user scope; an elevated Setup does not automatically give its token to the recorder. Physical UAC/all-users certification is not claimed. |
| UI interaction | Controller/media tests and real UIA client actions passed. Physical mouse/keyboard/folder-picker tests remain separate and pending. |
| Longer capture | One ten-minute 720p/30 capture completed, with CPU/RAM/disk sampling, file reopening/decoding and owned-media cleanup. This does not certify hours-long sessions or every quality profile. |
| AMD/Intel/other PCs | Not certified by this local environment. |
| Physical webcam | No camera was enumerated here. Camera layouts use labeled synthetic data; physical capture remains a required retest. |
| Physical DPI/multi-monitor | Renderer scales are covered; physical 125/150/200% DPI and multiple-monitor combinations remain untested. |
| Accessibility/UIA | Basic providers, names, bounds, navigation, focus, Invoke, Value, Toggle and RangeValue are implemented and tested. Complete TextPattern, screen-reader behavior and accessibility certification remain pending. |

## 0.6.3 local validation

The exact candidate passed the full 33-report pipeline: frozen offline build/reproducibility, 69 unit tests per application/Setup binary, short screen/audio capture, editing/export, recovery, language persistence, storage checks and current-user installer checks. The language gate checked 405 native renders, 65 checks and 1,498 explicit catalog entries in five languages. A client of the Windows UI Automation API exercised actual control actions in the application’s own process; this is distinct from physical input or an independent screen reader.

The ten-minute recording reserves disk space in addition to a media budget, logs samples, reopens/decodes the beginning/middle/end and records generated-file hashes before deletion. The application reserves space for recovery segments and the assembled MP4, checks the recording and journal destinations and reports unavailable space queries. No recovery guarantee is made after power loss, hardware failure or an external process unexpectedly consuming the disk.

## Known feature limits

- Basic effects and independent WAV mixing remain; video cuts do not automatically realign independent WAV tracks.
- Camera video remains composited, with no separate editable camera-video track.
- Exclusive game hooks and universal protected-content/anti-cheat capture are not implemented or certified.
- Recovery may lose the last open segment. Full keyframes and complete accessibility remain future work.

[0.6.3 notes](releases/0.6.3.md) · [Installation](INSTALLATION.md) · [Troubleshooting](TROUBLESHOOTING.md) · [Back to project](../README.md)
