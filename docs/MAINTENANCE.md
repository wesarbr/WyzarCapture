# Maintaining the project page and releases

This repository is the English distribution and documentation portal for WyzarCapture. Keep product documentation consistent with the installer actually attached to the official Release. Proprietary source code, development tools, private validation records, credentials and personal captures do not belong here.

## For each new version

1. Confirm the version, release channel, tag, exact tested installer and the features that changed. Keep implementation, local testing and hardware retesting distinct.
2. Update the README version, brief description, feature summary, installation links and current release-note link. Update CHANGELOG, installation, quick start, FAQ, troubleshooting, beta status and the bug-report version example where relevant. Keep previous changelog entries as history.
3. Check LICENSE and PRIVACY for version references or changed behavior. Preserve the custom Freeware License; do not select a different license template. Changing a version reference does not change the license terms.
4. Inspect images before replacing them. Refresh screenshots or artwork that display an obsolete version. Retain version-neutral logos, icons and illustrations unless their design or meaning changed. Use stable asset names so existing links survive.
5. Use the actual native application/Setup renderer with isolated demonstration preferences and synthetic media. Never publish private desktops, conversations, audio, photos or projects. Label synthetic camera content explicitly and do not present renders as physical-device tests.
6. Prepare concise release notes: user-visible improvements, corrections, installation/update steps, languages and material beta limits. Link to the usage guides. Do not copy internal audit tables or private checksum values into public text.
7. Create a versioned GitHub Release against the reviewed documentation commit. Mark beta builds as a **pre-release**. Attach only the official Setup executable when that is the approved distribution scope. Do not attach development files, audit reports, checksum files or extra archives.
8. Verify the published title, tag, pre-release state and executable asset. Download the uploaded installer and compare it with the tested local bytes in the private audit. Only then change download buttons to a direct versioned asset link and mark availability as verified.
9. Review the rendered README, release notes, images and relative links on GitHub. Preserve repository visibility unless the owner explicitly requests a change. Record publication evidence outside this portal.

## Version and image map

| Location | What to check |
| --- | --- |
| README and CHANGELOG | Current version, new behavior, release-note link and download destination. |
| docs/releases | A new version-specific note; preserve historical notes. |
| docs/BETA_STATUS.md | Actual distribution state, tested OS/devices and unresolved limits. |
| Installation, quick start, FAQ and troubleshooting | Current names, defaults, paths through the interface and recovery steps. |
| LICENSE, PRIVACY and the bug-report form | Version references, applicable behavior and useful report fields. |
| assets/github/hero.png and download.svg | Visible version badge, action wording and valid link from README. |
| Native interface images and languages.png | Footer/version labels and the current screen used by composite artwork. |
| Version-neutral logo and compact illustrations | Keep unchanged unless an update is explicitly needed; explain older illustrative layouts in the gallery. |
| GitHub About | A brief, accurate project description without version-specific claims that will quickly become stale. |

Search for the old version and obsolete “preview/pending download” wording, then assess each match. Historical changelog entries can retain their original version. Do not promise an installer is available until the Release asset exists and has been checked.

[Release notes](../CHANGELOG.md) · [Beta status](BETA_STATUS.md) · [Back to project](../README.md)
