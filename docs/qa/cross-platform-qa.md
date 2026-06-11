# Cross-Platform QA

AirBridge targets three operating system families. This document describes the supported platform configurations, the checks to perform on each, and known platform-specific behavior.

---

## Supported Platforms

| Platform | Architecture | Minimum Version |
|----------|-------------|----------------|
| macOS | Intel (x86_64) | macOS 12 Monterey |
| macOS | Apple Silicon (arm64) | macOS 12 Monterey |
| Windows | x86_64 | Windows 10 (21H2) |
| Windows | x86_64 | Windows 11 |
| Linux | x86_64 | Ubuntu 22.04 LTS, Debian 12 |

**Note:** 32-bit builds are not provided. ARM Linux builds are not currently provided.

---

## Install Smoke Tests Per Platform

### macOS (Intel and Apple Silicon)

- [ ] Download and mount the `.dmg`.
- [ ] Drag the application to `/Applications`.
- [ ] Launch from `/Applications` — Gatekeeper does not show an "unidentified developer" dialog.
- [ ] On Apple Silicon: confirm that the app runs natively (Activity Monitor shows "Apple" in the Kind column, not "Intel" indicating Rosetta).
- [ ] Uninstall by dragging to Trash — no background processes remain after a system restart.

### Windows 10 / 11

- [ ] Run the `.msi` or `.exe` installer as a standard (non-administrator) user, if the installer is designed for per-user install. If system-wide install is required, confirm the UAC elevation dialog is shown and correctly requests administrator rights.
- [ ] After installation, launch from the Start Menu shortcut.
- [ ] Confirm that the application appears in "Apps & features" for uninstall.
- [ ] Uninstall and verify that no files remain in `%LOCALAPPDATA%\AirBridge` or `%APPDATA%\AirBridge` (or document intentional persistence).

### Linux (Ubuntu 22.04 / Debian 12)

- [ ] Mark the `.AppImage` as executable (`chmod +x`) and launch it directly.
- [ ] Application launches without requiring additional shared libraries beyond what ships with Ubuntu 22.04 minimal desktop.
- [ ] If a `.deb` package is provided: `sudo dpkg -i airbridge-*.deb` installs without errors. Application appears in the application menu. `sudo apt remove airbridge` removes it cleanly.

---

## Window Sizing and Layout

- [ ] On a 1080p (1920×1080) display, all primary views fit without horizontal or vertical overflow that requires scrolling to access controls.
- [ ] On a 4K / HiDPI display (macOS Retina, Windows 150%+ scaling, Linux HiDPI), text and icons are sharp and not blurry.
- [ ] Resizing the application window to a minimum size does not break layout or cause text to overlap controls.
- [ ] Maximizing the application window does not cause excessive whitespace or misaligned layouts.
- [ ] On macOS: the traffic-light buttons (close/minimize/zoom) are visible and functional.
- [ ] On Windows: the title bar minimize/maximize/close buttons are visible and functional.

---

## Font Rendering

- [ ] On macOS: body text uses the system font (San Francisco) or a bundled font that renders cleanly at Retina resolution.
- [ ] On Windows: text renders with ClearType subpixel antialiasing at standard DPI. On HiDPI displays, text is sharp.
- [ ] On Linux: text is legible at the default system font size. Emoji in log messages or error strings render without broken glyph boxes.
- [ ] Monospace text (file paths, JSON previews, record IDs) uses a legible monospace font on all platforms.

---

## File Dialog Behavior

- [ ] On macOS: "Open" and "Save" dialogs are the native macOS panels (not web-based alternatives).
- [ ] On Windows: dialogs are the native Windows Explorer-style panels.
- [ ] On Linux: dialogs are the native GTK or KDE file picker, depending on the desktop environment.
- [ ] The default starting directory for "Save backup" is the user's home directory or Documents folder, not the application's installation directory.
- [ ] File extension filters are applied correctly (e.g., only `.airbridge` packages are selectable in "Open backup" dialogs).
- [ ] Canceling a file dialog does not produce an error message — the application silently returns to its previous state.

---

## Path Separator Handling

- [ ] On Windows: paths displayed in the UI use backslashes (`\`) for local paths, consistent with Windows conventions.
- [ ] On macOS and Linux: paths use forward slashes (`/`).
- [ ] Backup packages created on Windows can be opened on macOS/Linux, and vice versa. Internal path references within the package (if any) use forward slashes as a portable convention.
- [ ] The app data directory path shown in Settings is correct and OS-appropriate (see next section).

---

## App Data Directory Location

The application stores configuration and logs in the standard per-platform location:

| Platform | Expected path |
|----------|---------------|
| macOS | `~/Library/Application Support/AirBridge/` |
| Windows | `%APPDATA%\AirBridge\` (i.e., `C:\Users\<user>\AppData\Roaming\AirBridge\`) |
| Linux | `~/.config/AirBridge/` or `$XDG_CONFIG_HOME/AirBridge/` if `XDG_CONFIG_HOME` is set |

- [ ] On macOS: data directory exists at the expected path after first launch.
- [ ] On Windows: data directory exists at the expected path after first launch.
- [ ] On Linux: data directory exists at the expected path (respecting `XDG_CONFIG_HOME`) after first launch.
- [ ] The path displayed in the Settings view matches the actual filesystem path.

---

## Known Platform-Specific Limitations

| Platform | Limitation | Workaround |
|----------|-----------|------------|
| Linux | Native file dialogs require `zenity` or `kdialog` to be installed on minimal desktop environments | Document this in the Linux installation instructions |
| Windows | File paths longer than 260 characters may cause issues when saving backups to deeply nested directories | Use a shorter output path |
| macOS (Intel) | Rosetta translation is not required but will be used if the Intel binary is run on Apple Silicon without the universal binary | Always distribute the universal binary or both architecture-specific binaries |
| Windows 10 | Acrylic/Mica window effects are not available; the window uses a flat background | Expected behavior, not a defect |
