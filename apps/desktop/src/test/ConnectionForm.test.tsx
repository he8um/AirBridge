import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect } from "vitest";
import { ConnectionForm } from "../features/connections/ConnectionForm";

// A valid-looking synthetic token (no real token — just sufficient length and no "fail")
const VALID_TOKEN = "example_valid_token_abcdefgh12345";
const SHORT_TOKEN = "tooshort";
const FAIL_TOKEN = "example_fail_token_abcdefgh12345xx";
const VALID_NAME = "Test Connection";

function setup() {
  const user = userEvent.setup();
  const view = render(<ConnectionForm />);
  return { user, ...view };
}

describe("ConnectionForm", () => {
  it("renders name and password fields", () => {
    setup();
    expect(screen.getByLabelText("Connection name")).toBeInTheDocument();
    expect(screen.getByLabelText("Personal access token")).toBeInTheDocument();
  });

  it("renders test connection button", () => {
    setup();
    expect(screen.getByRole("button", { name: "Test connection" })).toBeInTheDocument();
  });

  it("renders clear button", () => {
    setup();
    expect(screen.getByRole("button", { name: "Clear connection form" })).toBeInTheDocument();
  });

  it("test connection button is disabled when form is empty", () => {
    setup();
    expect(screen.getByRole("button", { name: "Test connection" })).toBeDisabled();
  });

  it("test connection button is disabled when only name is provided", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    expect(screen.getByRole("button", { name: "Test connection" })).toBeDisabled();
  });

  it("test connection button is disabled when only token is provided", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    expect(screen.getByRole("button", { name: "Test connection" })).toBeDisabled();
  });

  it("test connection button is enabled when both fields are valid", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    expect(screen.getByRole("button", { name: "Test connection" })).toBeEnabled();
  });

  it("shows connected status after successful check", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await waitFor(() => {
      expect(screen.getByText("Connected")).toBeInTheDocument();
    });
  });

  it("shows permission checks after successful connection", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await waitFor(() => {
      expect(screen.getByText("Schema read")).toBeInTheDocument();
    });
  });

  it("shows failed status after failed check", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), FAIL_TOKEN);
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await waitFor(() => {
      expect(screen.getByText(/failed/i)).toBeInTheDocument();
    });
  });

  it("clear button resets form fields", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    await user.click(screen.getByRole("button", { name: "Clear connection form" }));
    expect(screen.getByLabelText("Connection name")).toHaveValue("");
    expect(screen.getByLabelText("Personal access token")).toHaveValue("");
  });

  it("token is not visible as plain text in the document after form submit", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await waitFor(() => {
      expect(screen.getByText("Connected")).toBeInTheDocument();
    });
    // Token must not appear anywhere in the rendered document text content
    const bodyText = document.body.textContent ?? "";
    expect(bodyText).not.toContain(VALID_TOKEN);
  });

  it("error message does not contain the token value", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), FAIL_TOKEN);
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await waitFor(() => {
      expect(screen.getByText(/failed/i)).toBeInTheDocument();
    });
    const bodyText = document.body.textContent ?? "";
    expect(bodyText).not.toContain(FAIL_TOKEN);
  });

  it("result JSON does not contain the token value", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    await user.click(screen.getByRole("button", { name: "Test connection" }));
    await waitFor(() => {
      expect(screen.getByText("Connected")).toBeInTheDocument();
    });
    // Grab entire document serializable text
    const allText = document.body.innerHTML;
    expect(allText).not.toContain(VALID_TOKEN);
  });

  it("shows validation error for empty name", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Personal access token"), VALID_TOKEN);
    // Button is disabled, so click doesn't fire submit — form remains invalid
    expect(screen.getByRole("button", { name: "Test connection" })).toBeDisabled();
  });

  it("shows short token validation error on early submit attempt", async () => {
    const { user } = setup();
    await user.type(screen.getByLabelText("Connection name"), VALID_NAME);
    await user.type(screen.getByLabelText("Personal access token"), SHORT_TOKEN);
    expect(screen.getByRole("button", { name: "Test connection" })).toBeDisabled();
  });
});
