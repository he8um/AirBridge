export interface ValidationError {
  field: string;
  message: string;
}

export interface ConnectionFormInput {
  name: string;
  token: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
}

export function validateConnectionName(name: string): ValidationError | null {
  const trimmed = name.trim();
  if (trimmed.length === 0) {
    return { field: "name", message: "Connection name is required." };
  }
  if (trimmed.length < 2) {
    return { field: "name", message: "Connection name must be at least 2 characters." };
  }
  if (trimmed.length > 80) {
    return { field: "name", message: "Connection name must be 80 characters or fewer." };
  }
  return null;
}

export function validatePersonalAccessToken(token: string): ValidationError | null {
  if (token.length === 0) {
    return { field: "token", message: "Personal access token is required." };
  }
  if (token.trim().length === 0) {
    return { field: "token", message: "Personal access token must not be whitespace only." };
  }
  if (token.length < 20) {
    return { field: "token", message: "Token appears too short to be valid." };
  }
  return null;
}

export function validateConnectionForm(input: ConnectionFormInput): ValidationResult {
  const errors: ValidationError[] = [];
  const nameError = validateConnectionName(input.name);
  if (nameError) errors.push(nameError);
  const tokenError = validatePersonalAccessToken(input.token);
  if (tokenError) errors.push(tokenError);
  return { valid: errors.length === 0, errors };
}
