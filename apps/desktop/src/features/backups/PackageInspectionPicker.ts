/**
 * Opens a native file picker filtered to `.airbridge` files.
 * Returns the selected absolute path, or null if the user cancelled.
 *
 * This module is the only place in the inspection flow that receives a full
 * path. The path is passed directly to the Tauri command and never stored
 * or rendered in the UI — only the filename returned by the command is shown.
 */
export async function pickBackupPackagePath(): Promise<string | null> {
  try {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({
      multiple: false,
      filters: [{ name: "AirBridge Package", extensions: ["airbridge"] }],
    });
    if (typeof selected === "string" && selected.length > 0) {
      return selected;
    }
    return null;
  } catch {
    return null;
  }
}
