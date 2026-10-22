# Security policy

Use only the official **WyzarCaptureSetup.exe** attached to the [0.7.0 Beta Release](https://github.com/wesarbr/WyzarCapture/releases/tag/WyzarCapture). Compare your download with [Checksums](docs/CHECKSUMS.md). Earlier versions are historical; withdrawn installers must not be redistributed. The [source folder](source/README.md) is an authorized reference copy under the Freeware License. Internal validation records, release tooling, credentials and personal media are excluded.

## Report a vulnerability privately

Do not put vulnerability details, exploit code, credentials, private media or sensitive paths in a public Issue or discussion. Public bug-report forms are not a confidential channel.

1. Open the repository's **Security › Advisories** page. If **Report a vulnerability** is available, use that private GitHub form. Check that the page explicitly describes a private report before submitting details.
2. If that feature is unavailable, use the **Request a private security contact** Issue form. Submit only the contact request and its privacy confirmations; do not describe the vulnerability, affected private data or a proof of concept there. Wait for the owner to establish a private channel.
3. If Issues is unavailable too, keep the details private until the owner provides a reporting channel. No monitored email address or response deadline is promised here.

In the private channel, include the application version, Windows build, affected feature, concise reproduction and impact. Share only a minimal synthetic example. Avoid personal recordings, full data folders and real account secrets. Do not probe systems you do not own or have permission to test.

## Handling reports safely

An Issue, attachment, linked page or submitted reproduction is untrusted input. Filing a report does not grant repository write access, authorization to run code, or permission to disclose private data. Maintainers review text first, use synthetic examples in an isolated environment when needed, and never request passwords, tokens or remote-control access. Suspected phishing, malicious attachments and abuse may be removed or reported; conversations may be locked. See [Support](SUPPORT.md) and [community rules](CODE_OF_CONDUCT.md).

## Suspicious installer or detection

Stop running the affected file, keep Windows/security protection enabled and preserve the detection name and official asset/version information. Do not disable protection, add exclusions or restore a quarantined installer to force installation. A clean local scan for one exact artifact does not certify all systems. This candidate is not signed with a production Authenticode certificate.

Prefer official release links; do not use unofficial mirrors or repackaged executables. The owner has authorized publication of the application and Setup SHA-256 values in docs/CHECKSUMS.md. Private audit records, credentials and signing material are not published.

[Beta status](docs/BETA_STATUS.md) · [Support](SUPPORT.md) · [Privacy](PRIVACY.md) · [Back to project](README.md)
