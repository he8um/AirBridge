# Installation

AirBridge is distributed as a desktop application for macOS, Windows, and Linux.

## Release artifacts

Planned release assets:

```text
AirBridge-x.y.z-macos.dmg
AirBridge-x.y.z-windows.msi
AirBridge-x.y.z-linux.AppImage
AirBridge-x.y.z-linux.deb
checksums.txt
```

## macOS

1. Download the `.dmg` release asset.
2. Open the disk image.
3. Drag AirBridge into Applications.
4. Open AirBridge.

Early unsigned builds may require additional confirmation in macOS security settings.

## Windows

1. Download the `.msi` or `.exe` release asset.
2. Run the installer.
3. Follow the installation steps.
4. Open AirBridge from the Start menu.

Early unsigned builds may show a warning. Verify the release source and checksum before installing.

## Linux

### AppImage

1. Download the `.AppImage` file.
2. Mark it executable:

```bash
chmod +x AirBridge-x.y.z-linux.AppImage
```

3. Run it:

```bash
./AirBridge-x.y.z-linux.AppImage
```

### Debian package

1. Download the `.deb` file.
2. Install it:

```bash
sudo dpkg -i AirBridge-x.y.z-linux.deb
```

3. Resolve dependencies if needed:

```bash
sudo apt-get install -f
```

## Verifying checksums

Download `checksums.txt` from the release and compare the hash of the installer you downloaded.

Example:

```bash
sha256sum AirBridge-x.y.z-linux.AppImage
```

The output should match the corresponding value in `checksums.txt`.
