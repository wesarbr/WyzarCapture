# Make your first recording

## Choose language and destinations

Open **Settings › General › Language** to choose your interface language. This guide uses English control names. In **Settings › Storage**, confirm your recording, audio, screenshot, export and project destinations. The initial profile is a starting point, not a performance guarantee.

## Choose what to capture

- **Full screen:** select a detected monitor in Capture settings.
- **Window:** select an open window.
- **Region:** drag an area in the native desktop selector and confirm it.
- **Game:** select the game window. Windowed or borderless mode is recommended for this capture path; protected content and some games can prevent capture.

The detected monitor preview starts automatically during normal launch. Use the preview control to check framing for the selected source. Preview rendering preserves the source aspect ratio. Content excluded by Windows or protected by another application may not appear.

## Set picture and sound

Choose resolution, frame rate and bitrate appropriate to your computer. A requested rate does not prove that every submitted frame is unique or that all systems will sustain it. Use a lower profile if needed.

Enable microphone and system sound independently. Select real available endpoints and confirm their levels. Recording modes include video with mixed audio, silent video, screen with companion WAV files, and audio-only WAV recording. Camera is optional; check Windows camera permissions and close competing camera applications if it cannot start.

## Record and verify

Start with the record button or F8. The countdown appears on the capture monitor. F9 pauses/resumes; F10 stops. Wait for finalization and the saved notice before closing the application. These shortcuts are configurable. Stop a short test and reopen it in the Library to confirm picture, sound, framing and destination.

### Take a screenshot

Open **Screenshot** or press **PrintScreen**. Choose the recording source, a monitor, an open window or a region, then select the target. Choose a delay of 0, 3, 5 or 10 seconds, whether to include the cursor and whether to copy the image for pasting. Use **Capture now** to save a PNG at the source dimensions, independently of the recording resolution. The saved notice opens the screenshot folder. The same controls are available from compact layouts.

### See your camera and reach your files

When a webcam is detected, open its expanded preview to check the complete image with its aspect ratio preserved. Previewing alone does not add the camera to a recording; enable camera recording explicitly. Camera controls are hidden when none is detected.

Open **Your folders** to open or change a destination. Keep the Windows folders by file type, use Restore Windows folders, or explicitly choose custom writable destinations. There is no automatic AppData media fallback. Previous files stay in their existing locations.

### Personalize keys

In **Settings › Shortcuts**, click an action and press a key or a combination with Ctrl, Alt, Shift or Win. Confirm the assignment or restore F8/F9/F10/PrintScreen. Duplicates are rejected. Windows-reserved or already-used shortcuts may require another combination. The screenshot shortcut opens its current controls; **Capture now** applies the selected settings.

A captured window that closes can interrupt recording; check Recovery for completed segments, without assuming all content can be restored.

## Organize and edit

Use the Library's search, tags and favorites to find a recording. Renaming or deletion groups recognized companion files; inspect the deletion confirmation. Importing an external file keeps its original location, so keep it available.

Open the editor, trim or split clips and remove unwanted parts. Adjust speed, volume and basic effects, then inspect the preview. Associated/imported WAV tracks have separate mix controls for mute, solo, gain, offset, fades and cuts. Video cuts do not automatically realign those tracks. Webcam remains composited into the recorded image.

Save a `.wyzproj` project to preserve editing decisions. It references original media rather than embedding every source. Move or delete those sources only if you are prepared to relink/rebuild the project.

## Storage and accessibility since 0.6.3

Keep room for the segment files and the assembled MP4. The app checks space before and during recording, audio capture and export. If space is insufficient or cannot be queried, review the message and choose another writable destination. Completed recovery segments are preserved if assembly cannot finish.

Supported controls expose names, focus and Invoke/Value/Toggle/RangeValue actions to Windows UI Automation. This is basic interoperability, not complete screen-reader certification.

## Export

Choose a name, writable destination, resolution, frame rate and bitrate. Export creates another H.264/AAC MP4. Reopen the exported file and check the entire portion you intend to share. Basic blur is not automatic sensitive-data detection.

[Troubleshooting](TROUBLESHOOTING.md) · [Language controls](LANGUAGES.md) · [Beta limits](BETA_STATUS.md) · [Back to project](../README.md)
