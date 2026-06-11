import type { AirBridgeCommandError } from "./types";

export function isAirBridgeCommandError(value: unknown): value is AirBridgeCommandError {
  return (
    typeof value === "object" &&
    value !== null &&
    "code" in value &&
    "message" in value &&
    typeof (value as AirBridgeCommandError).code === "string" &&
    typeof (value as AirBridgeCommandError).message === "string"
  );
}

export function formatCommandError(err: unknown): string {
  if (isAirBridgeCommandError(err)) {
    return `[${err.code}] ${err.message}`;
  }
  if (typeof err === "string") return err;
  return "An unexpected error occurred.";
}
