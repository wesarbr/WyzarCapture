# Executable checksums

These SHA-256 values identify the exact locally tested files. They do not provide Authenticode signing or universal security certification. Compare the installer before use and the installed application with its version below.

## Corrected 0.6.3 Beta candidate

Prepared locally; not yet attached to a GitHub Release. Do not confuse it with the existing 0.6.1 download.

| Executable | SHA-256 |
| --- | --- |
| WyzarCaptureSetup.exe | `41552B2C069A8F620E3DDCB6D5E0FFC57540C3BDDF36D651D7980A5F8618BC38` |
| WyzarCapture.exe (installed application) | `962B601C890A6E2D14C6911F7B4102AD1DEEC99A69DD5BCA1ACD7859BA7EBBC9` |

## Historical 0.6.1 Beta download

The existing [Release](https://github.com/wesarbr/WyzarCapture/releases/tag/WyzarCapture) has reported Windows folder issues. Its asset digest matches the Setup value below; this observation is distinct from downloading and comparing it again. These bytes do not include the 0.6.3 fixes.

| Executable | SHA-256 |
| --- | --- |
| WyzarCaptureSetup.exe | `F8D4BB248E92766418B04FC73094E6347FB6BEB2A61548EF583F8D7529188367` |
| WyzarCapture.exe (installed application) | `B57A3D7D779984902E16B250957FEB8E15C53C96D057C84256A7C272A292FF85` |

In PowerShell, use Get-FileHash with -Algorithm SHA256 and the exact file path. Only the Setup EXE is required for installation.

[Beta status](BETA_STATUS.md) · [Installation](INSTALLATION.md) · [Back to project](../README.md)
