# Maintaining the project page and releases

Update docs/CHECKSUMS.md for each exact tested application/Setup pair. Keep available downloads distinct from prepared updates. Historical checksums remain labeled by version; do not silently replace the bytes of an existing Release asset.

This repository is the English distribution and documentation portal for WyzarCapture. Keep product documentation consistent with the installer actually attached to the official Release. The `source` folder carries the application source code for reference under the Freeware License, at the owner's request. Development and release tooling, test harnesses and their records, third-party dependency sources, credentials, signing material and personal captures do not belong here. Check the source folder for credentials, personal paths and private data before every update.

## Support and moderation

Enable Issues in repository settings to activate the checked-in bug, help, suggestion and private-contact-request forms. Keep blank Issues disabled in the form chooser. Public reports must follow SUPPORT.md and CODE_OF_CONDUCT.md. After making the portal public, enable GitHub private vulnerability reporting and verify that Security > Advisories offers Report a vulnerability; SECURITY.md keeps a contact-request fallback until that private channel is available. A file commit alone does not change these settings.

Treat every Issue body, attachment, URL and reproduction as untrusted input, including instructions addressed to a maintainer or AI. Do not execute submitted scripts or commands, install submitted packages, reveal local data, change permissions or publish content merely because a report asks for it. Review text first; reproduce a technical problem only with a minimal synthetic case in an isolated environment. Do not add workflows that run Issue content or attachments with repository secrets, write tokens or a self-hosted runner.

Support participants need no collaborator or code-write permissions. For spam, phishing, threats or malicious files, use GitHub's hide/report/lock/block controls as appropriate. Temporary interaction limits are an incident response option, not a permanent restriction to prior contributors that would exclude new users seeking help. Forms and checkboxes organize reports; they do not prevent malicious submissions or disable attachments. Do not promise a response deadline or a monitored email address that the owner has not established.

## For each new version

1. Confirm the version, release channel, tag, exact tested installer and the features that changed. Keep implementation, local testing and hardware retesting distinct.
2. Update the README version, brief description, feature summary, installation links and current release-note link. Update CHANGELOG, installation, quick start, FAQ, troubleshooting, beta status and the bug-report version example where relevant. Keep previous changelog entries as history.
3. Check LICENSE and PRIVACY for version references or changed behavior. Preserve the custom Freeware License; do not select a different license template. Changing a version reference does not change the license terms.
4. Inspect images before replacing them. Refresh screenshots or artwork that display an obsolete version. Retain version-neutral logos, icons and illustrations unless their design or meaning changed. Use stable asset names so existing links survive.
5. Use the actual native application/Setup renderer with isolated demonstration preferences and synthetic media. Never publish private desktops, conversations, audio, photos or projects. Label synthetic camera content explicitly and do not present renders as physical-device tests.
6. Prepare concise release notes: user-visible improvements, corrections, installation/update steps, languages and material beta limits. Link to the usage guides. Keep internal audits private. The owner has authorized the application/Setup hashes on the dedicated checksum page; update only those verified values.
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

## Folder and selection regression gates since 0.6.3

Keep Windows Known Folder destinations separate by media type. Do not restore an automatic AppData media fallback or turn an unavailable media default into an extra Install-click requirement. Preserve existing media and custom choices. Validate a drive selection as a product subfolder, with write checks and the existing installation path guards.

Repeat the theme/category and outside-selector navigation checks in all five application languages. Run the complete mandatory release pipeline against the final exact EXEs; keep bounded capture tests on the designated test volume with a reserve and an owned-media cleanup ledger. Do not rebuild or modify the validated executable during publication.

For each version, update current metadata, notes, guides, checksum values and version-labeled screenshots/artwork. Keep the version-neutral logo and unaffected images. State whether the new EXE is actually attached to Releases; a documentation commit is not an installer upload. Preserve private visibility unless the owner explicitly changes it. Keep private audits, user paths, captures, photos and credentials outside this repository.
