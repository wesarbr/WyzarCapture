# Troubleshooting

## Recording cannot create a destination

Open **Settings › Storage**, choose an available local writable folder using the native picker, and retry a short recording. Setup and first launch create the media directory tree and test writing. Redirected or disconnected folders, permission restrictions and stale paths copied from another computer can still need correction. Free disk space alone does not prove a destination is available.

If it fails, use the operation's diagnostic copy action. Record the OS error number and which action failed. Remove personal paths before sharing screenshots or manually copied error text. A failed new folder request should not replace saved working preferences.

## A monitor or audio device is missing

Check that the device is available in Windows and connected in the current desktop session. Reopen the appropriate selector when idle. Device/display changes and the start of recording refresh inventory and reconcile stale selections. A failure in one category should leave other categories available.

Check microphone permissions and the selected input/output endpoints. System audio follows the chosen playback endpoint; an unused endpoint can produce silence. Reconnect devices while idle and retry a short test.

## Camera cannot start

Confirm Windows camera access for desktop applications, check the driver and close other applications that may be holding the camera. Select the listed camera explicitly and retry while idle. The candidate tries bounded native formats and advanced/legacy processing paths, validates buffer dimensions/stride and preserves source ownership while retrying.

If the error persists, disable the camera temporarily to test screen recording independently. Report the camera model, Windows build, selected mode, stage and numeric HRESULT. The Windows 10 notebook's physical camera remains a required retest; these changes are not a camera compatibility certification.

## Recording is slow or drops frames

Reduce resolution/frame rate, close competing workloads, check free space and use a suitable local destination. A selected quality profile and format negotiation do not prove sustained capture rate. Hardware encoding availability depends on drivers and media transforms. Verify actual exported playback.

## A window or game is black

Try a normal visible window or windowed/borderless game. Protected content and some application/game rendering paths can prevent Windows Graphics Capture. An exclusive game capture hook is not provided.

## Language, fonts or layout

Use **Settings › General › Language**, or the installer's header selector. Language names remain recognizable as autonyms. Chinese is Simplified. Windows dialogs/system messages can remain in the Windows language; file and device names are data and are preserved. Report missing glyphs or clipping with your selected language and scaling. See the [language guide](LANGUAGES.md).

## An interrupted recording or missing project source

Check Recovery for completed segments. Preserve originals if an operation fails. Recovery may lose the last open segment and is not guaranteed after disk/power failure. Projects reference source media; restore the source location if it has moved or disappeared.

## Security software blocks Setup

Keep protection enabled. Do not add an exclusion or restore a quarantined file just to make installation proceed. Preserve the detection name and official asset/version information; follow the [security policy](../SECURITY.md). The candidate is unsigned.

[Installation](INSTALLATION.md) · [Quick start](QUICK_START.md) · [Back to project](../README.md)
