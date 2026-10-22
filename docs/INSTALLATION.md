# Installation and updates

**Prepared update: 0.6.3 Beta for Windows x64.** Its application and Setup have completed local validation but the new installer is not yet attached to a GitHub Release. The existing [0.6.1 Beta Release](https://github.com/wesarbr/WyzarCapture/releases/tag/WyzarCapture) has reported Windows folder issues and does not include these corrections. See [beta status](BETA_STATUS.md) and [exact executable checksums](CHECKSUMS.md).

## Install the corrected version

1. Use only the official **WyzarCaptureSetup.exe** for the intended version. This single executable contains the runtime; no separate payload, scripts or documents are needed.
2. Choose a language and read the license. Application, Setup, help and new messages support Brazilian Portuguese, English, Spanish, Simplified Chinese and Japanese.
3. Choose the application destination. Current-user installation accepts a writable subfolder on another local Windows drive. Selecting a drive root uses its WyzarCapture subfolder. Protected folders still require an available destination; the beta does not enable all-users/UAC installation.
4. Keep the Windows media defaults, or explicitly choose **Use a custom folder for all media**. The application folder and media folder are separate choices. Setup no longer sends media to AppData or requires another Install click to approve an automatic fallback.
5. Choose shortcuts and optional startup behavior, then install. Start the recorder from your normal Windows session. An elevated Setup does not automatically launch it with its elevated token.

## Default media destinations

| File type | Windows destination |
| --- | --- |
| Recordings | Videos / WyzarCapture |
| Screenshots | Pictures / WyzarCapture |
| Audio recordings | Music / WyzarCapture |
| Video exports | Videos / WyzarCapture / Exports |
| Editing projects | Documents / WyzarCapture / Projects |

These use the actual Windows Known Folder locations, including folder redirection. Cache, preferences, library indexes, logs and recovery working data remain in the application data location. If Windows cannot provide a media destination, choose another available writable folder in Storage. The application does not disable protection or silently redirect media to AppData.

**Your folders** provides open/change actions for every destination and **Restore Windows folders**. A custom shared root is also available. Only future files use a changed destination. Existing files stay in place.

## Update, repair and uninstall

Updating the complete old automatic AppData/Media layout changes future destinations to the Windows media folders and saves a copy of the previous preferences. Existing videos, screenshots, audio and projects are preserved. Other custom choices are preserved.

Use the official Setup for update or repair. Current-user installation, update, repair, rollback and uninstall have isolated local acceptance checks. Removal preserves personal media and modified legacy documents. Optional cleanup applies only to the explicitly selected recognized application data. See [tested scope](BETA_STATUS.md).

## Portable testing

The portable package keeps its preferences and cache beside the executable. New media uses the same Windows folder defaults as the installed version; choose custom destinations if you want all media on a particular drive. The portable package is not an additional dependency for Setup.

[Quick start](QUICK_START.md) · [Troubleshooting](TROUBLESHOOTING.md) · [Back to project](../README.md)
