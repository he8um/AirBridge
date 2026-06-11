# Code Signing and Notarization

## Purpose

Early releases may be unsigned. Stable releases should evaluate macOS notarization and Windows code signing to reduce installation warnings.

## Required release checks

- Changelog updated.
- Version updated.
- CI passing.
- Documentation reviewed.
- Known limitations reviewed.
- Build artifacts generated.
- Checksums generated.
- Release notes drafted.

## Artifact naming

```text
AirBridge-x.y.z-macos.dmg
AirBridge-x.y.z-windows.msi
AirBridge-x.y.z-linux.AppImage
AirBridge-x.y.z-linux.deb
checksums.txt
```

## Release note sections

```text
Summary
Added
Changed
Fixed
Security
Known limitations
Upgrade notes
Checksums
```
