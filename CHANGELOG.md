# Release notes

## 0.7.0 Beta — October 22, 2026

Redesigned installer that detects an existing installation in another folder and offers to keep or move it, ignores stale entries and keeps messages clear of the options. Cleaner interface with Segoe UI Variable, short transitions and an animated effect for each theme. On-demand redraw and graphics-device recovery. Variable bitrate with 2-second keyframes, recommended bitrate, antialiased scaling, parallel webcam reading with shapes and mirror, Enhance voice and a compatible capture path. [Notes](docs/releases/0.7.0.md) · [tested scope](docs/BETA_STATUS.md).

## 0.6.3 Beta — October 22, 2026

Historical release, superseded by 0.7.0: Windows media folders by type, removal of automatic AppData media fallback, drive-root selection into a product subfolder, and theme/menu hit-area and focus fixes. Includes disk reserves and basic UI Automation. All five languages updated. [Notes](docs/releases/0.6.3.md) · [current status](docs/BETA_STATUS.md).


## 0.6.2 Beta — October 22, 2026

Historical prepared candidate, superseded before distribution: recording/finalization disk reserve, audio/export space checks, basic Windows UI Automation and a bounded ten-minute capture gate. All new messages are embedded in five languages. See [0.6.2 notes](docs/releases/0.6.2.md) and [current status](docs/BETA_STATUS.md).


## 0.6.1 Beta — October 22, 2026

Configurable PrintScreen capture, key-based shortcut customization, quick access to every media destination, writable folder recovery, detected monitor preview and expanded camera viewing. Editor fixes preserve unsaved-change state after undo/redo and correct cumulative export timing across short cuts. New controls, Setup messages and help are included in all five languages.

Read the [full 0.6.1 notes](docs/releases/0.6.1.md), [installation guide](docs/INSTALLATION.md) and [current beta status](docs/BETA_STATUS.md). Official installers belong to [Releases](https://github.com/wesarbr/WyzarCapture/releases).


## 0.6.0 Beta candidate — October 22, 2026

Historical documentation candidate; superseded by 0.6.1. This entry does not describe current download availability.

### Languages

English, Spanish, Simplified Chinese, Japanese and Brazilian Portuguese are embedded in the native application and Setup. Language selectors persist the choice; a Setup request is applied once and later application changes are preserved. Help, update history, own messages and complete own freeware terms are translated. User content, real device names, Windows messages and original third-party notices retain their intended wording.

### First use and compatibility corrections

Setup separates application and media destinations, prepares available writable media folders and hands the choice to normal first launch. Initial recording defaults use detected hardware rather than a fixed quality assumption. Inventory keeps partial results and reconciles stale device selections. Camera configuration uses bounded formats, advanced/legacy negotiation, RGB32 stride/orientation checks and corrected Media Foundation source ownership during retries.

### Validation boundaries

The Windows 10 notebook's reported recording and physical camera errors require a new test on that computer. Local native rendering and short media tests do not certify other hardware, physical DPI, protected games or hours-long operation. The candidate remains unsigned. See [beta status](docs/BETA_STATUS.md).

## Existing workflow

The beta foundation includes Windows Graphics Capture, WASAPI audio, H.264/AAC MP4, PNG screenshots, local preferences/profiles, a library, basic editing/export, independent WAV mix controls, themes, compact layouts, local help and segmented recovery attempts. These features retain their documented limits. Earlier local candidates do not substitute for approval of this version.

[Back to project](README.md)
