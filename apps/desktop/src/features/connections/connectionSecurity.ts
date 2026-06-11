const REDACTED_PLACEHOLDER = "[redacted]";

export function redactSecret(value: string): string {
  if (value.length === 0) return value;
  return REDACTED_PLACEHOLDER;
}

export function hasSecretLeak(text: string, secret: string): boolean {
  if (secret.length === 0) return false;
  return text.includes(secret);
}

export function sanitizeConnectionError(error: unknown, secret?: string): string {
  let message: string;

  if (error instanceof Error) {
    message = error.message;
  } else if (typeof error === "string") {
    message = error;
  } else {
    message = "Connection check failed. Please try again.";
  }

  if (secret && secret.length > 0 && message.includes(secret)) {
    message = message.split(secret).join(REDACTED_PLACEHOLDER);
  }

  return message;
}
