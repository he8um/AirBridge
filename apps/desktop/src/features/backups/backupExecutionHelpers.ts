/** Required confirmation text that the caller must supply verbatim. */
export const BACKUP_CONFIRMATION_TEXT = "CREATE BACKUP";

/**
 * Extract the filename component from a path string.
 * Works with both forward-slash (macOS/Linux) and backslash (Windows) separators.
 * Returns an empty string if the path is empty or ends with a separator.
 */
export function getDisplayFileName(path: string): string {
  if (!path) return "";
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/");
  return parts[parts.length - 1] ?? "";
}

/** Return true if the path ends with the `.airbridge` extension. */
export function hasAirbridgeExtension(path: string): boolean {
  return path.endsWith(".airbridge");
}

/**
 * Return a redacted representation of a path that shows only the filename.
 * The full directory is replaced with "…" so it is never rendered in the UI.
 */
export function redactOutputPath(path: string): string {
  const filename = getDisplayFileName(path);
  if (!filename) return "";
  return `…/${filename}`;
}

/**
 * Build the confirmation instruction text shown to the user.
 * The user must type the returned phrase to unlock backup execution.
 */
export function buildConfirmationText(baseName: string): string {
  void baseName;
  return BACKUP_CONFIRMATION_TEXT;
}
