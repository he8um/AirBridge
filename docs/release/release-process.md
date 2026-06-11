# Release Process

## Purpose

Releases should be prepared from a release branch or tag, pass CI, build platform artifacts, generate checksums, and publish release notes with known limitations.

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
