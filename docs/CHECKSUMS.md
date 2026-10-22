# Executable checksums

These SHA-256 values identify the exact locally tested files. They do not provide Authenticode signing or universal security certification. Compare the installer before use and the installed application with its version below.

## 0.7.0 Beta

Exact locally validated candidate; installer upload is pending. These values identify the prepared files and do not imply that 0.7.0 is downloadable. See [release status](BETA_STATUS.md).

| Executable | SHA-256 |
| --- | --- |
| WyzarCaptureSetup.exe | `223DD7779A4F2C0EBC453CF426C5FDF72865E296337FFAF4D7386278608B8326` |
| WyzarCapture.exe (installed application) | `FB4F83DE0C0B17571C2288FAA675D25EDC159C34ED6AD38DAF5810FB0E8F0CB2` |

## Earlier versions

The existing [Release](https://github.com/wesarbr/WyzarCapture/releases/tag/WyzarCapture) currently contains the 0.6.3 Beta Setup. Its service-reported digest and size match the local 0.6.3 installer. The 0.6.1 values are historical; choose the row matching your file's version.

| Version | Executable | SHA-256 |
| --- | --- | --- |
| 0.6.3 Beta | WyzarCaptureSetup.exe | `41552B2C069A8F620E3DDCB6D5E0FFC57540C3BDDF36D651D7980A5F8618BC38` |
| 0.6.3 Beta | WyzarCapture.exe | `962B601C890A6E2D14C6911F7B4102AD1DEEC99A69DD5BCA1ACD7859BA7EBBC9` |
| 0.6.1 historical | WyzarCaptureSetup.exe | `F8D4BB248E92766418B04FC73094E6347FB6BEB2A61548EF583F8D7529188367` |
| 0.6.1 historical | WyzarCapture.exe | `B57A3D7D779984902E16B250957FEB8E15C53C96D057C84256A7C272A292FF85` |

In PowerShell, use Get-FileHash with -Algorithm SHA256 and the exact file path. Only the Setup EXE is required for installation.

[Beta status](BETA_STATUS.md) · [Installation](INSTALLATION.md) · [Back to project](../README.md)
