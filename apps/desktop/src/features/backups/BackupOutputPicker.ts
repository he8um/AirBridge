/**
 * Open the native OS save dialog and return the selected path, or null if the
 * user cancelled. Uses the Tauri dialog plugin when available; returns null in
 * jsdom / browser environments where the plugin is not present.
 *
 * No file is written by this function. It only returns a path string.
 */
export async function pickBackupOutputPath(defaultFileName?: string): Promise<string | null> {
  try {
    const { save } = await import("@tauri-apps/plugin-dialog");
    const result = await save({
      defaultPath: defaultFileName ?? "backup.airbridge",
      filters: [{ name: "AirBridge Package", extensions: ["airbridge"] }],
    });
    return result ?? null;
  } catch {
    return null;
  }
}
