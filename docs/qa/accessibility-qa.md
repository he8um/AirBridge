# Accessibility QA

AirBridge aims to be usable without a mouse and with common screen reader software. This document describes the accessibility checks to perform on each release.

---

## Keyboard Navigation

### Tab Order

- [ ] Pressing Tab from the first interactive element on any view cycles through all interactive elements in a logical, top-to-bottom, left-to-right order.
- [ ] No interactive element is skipped by Tab (unless it is visually hidden or disabled).
- [ ] Pressing Shift+Tab moves focus backward through the same sequence.
- [ ] Modals and dialogs trap focus correctly: Tab does not leave the dialog while it is open, and focus returns to the triggering element when the dialog is closed.

### Focus Ring

- [ ] A visible focus ring is present on every focused interactive element (buttons, inputs, links, dropdown triggers).
- [ ] The focus ring meets WCAG 2.1 SC 2.4.7 (Non-text Contrast): it has a contrast ratio of at least 3:1 against the adjacent background.
- [ ] The focus ring is not suppressed globally — `outline: none` is only used when a custom visible focus style replaces it.
- [ ] Focus is not lost to `document.body` when elements are removed from the DOM (e.g., after a dialog closes).

### Enter and Space Activation

- [ ] Buttons activate on Enter and Space.
- [ ] Links activate on Enter.
- [ ] Checkboxes toggle on Space.
- [ ] Select/dropdown controls open on Enter or Space and close on Escape.
- [ ] Activating "Start Backup" or "Start Restore" via keyboard produces the same result as clicking with a mouse.

### Escape Key

- [ ] Pressing Escape closes any open modal, drawer, or dropdown.
- [ ] Pressing Escape during a backup or restore in progress opens a cancel confirmation (or directly cancels, according to the design).
- [ ] Pressing Escape does not cause any data loss without confirmation.

---

## Screen Reader Support

### Platforms and Tools

| Platform | Screen Reader | Minimum version to test with |
|----------|--------------|------------------------------|
| Windows | NVDA | Latest stable |
| Windows | JAWS | Latest stable (if available) |
| macOS | VoiceOver | Built-in (macOS 12+) |
| Linux | Orca | Available in Ubuntu 22.04 |

### Checks

- [ ] **Page/view title announced.** When navigating to a new view (e.g., Backup, Restore, Settings), the screen reader announces the view title.
- [ ] **Button labels announced.** Every button is announced with a meaningful label (not just "button" or an icon name like "chevron").
- [ ] **Input labels announced.** Every text input is announced with its label when focused. Placeholder text alone does not count as a label.
- [ ] **Error messages announced.** When a form validation error or operation error occurs, the error text is announced. It is not left as a visually-styled message that the screen reader ignores.
- [ ] **Progress updates announced.** During a backup or restore operation, progress updates are announced at a reasonable frequency (not on every 1% tick, but periodically enough to be useful).
- [ ] **Status changes announced.** When an operation completes or fails, the completion or failure message is announced without requiring the user to navigate to it.
- [ ] **Table data readable.** Any tabular data (e.g., a list of tables in a base, a field list) is navigable row by row and column by column with the screen reader's table navigation commands.

---

## Heading Hierarchy

- [ ] The page has a single `<h1>` that describes the current view.
- [ ] Sub-sections use `<h2>` and `<h3>` in correct nesting order (no heading levels are skipped).
- [ ] Headings are used for structural navigation, not for visual styling of non-heading text.
- [ ] A screen reader's heading navigation mode allows the user to reach all major sections of each view.

---

## ARIA Labels and Roles

- [ ] Icon-only buttons have an `aria-label` that describes their action (e.g., `aria-label="Delete connection"`).
- [ ] Progress bars use `role="progressbar"` with `aria-valuenow`, `aria-valuemin`, and `aria-valuemax`.
- [ ] Loading spinners include an `aria-label` (e.g., `aria-label="Loading bases…"`) and are not announced repeatedly as the spinner animates.
- [ ] Lists of connections, tables, or fields use `<ul>`/`<li>` or appropriate ARIA list roles so that screen readers announce item counts.
- [ ] Dialogs use `role="dialog"` with `aria-labelledby` pointing to the dialog's title element.
- [ ] Status/alert messages use `role="status"` (for non-urgent updates) or `role="alert"` (for errors), so they are announced without requiring focus.

---

## Color Contrast

- [ ] Normal-sized body text meets WCAG AA (4.5:1 contrast ratio against background).
- [ ] Large text (18pt+ or 14pt+ bold) meets WCAG AA (3:1 contrast ratio).
- [ ] Interactive component boundaries (input borders, button outlines) meet 3:1 against adjacent colors.
- [ ] Status indicators (e.g., "Active" badge, error state highlight) do not rely solely on color to convey meaning — a label, icon, or pattern is also present.
- [ ] Contrast is verified in both light mode and dark mode (if both are supported).
- [ ] Verify contrast with the axe-core DevTools browser extension or a dedicated contrast checker tool.

---

## Reduced Motion

- [ ] When the operating system's "Reduce Motion" accessibility setting is enabled, AirBridge does not play animated transitions, spinning loaders, or sliding panels that could trigger vestibular discomfort.
- [ ] Progress indicators in reduced-motion mode use a static or non-animating style (e.g., a numeric percentage instead of a spinning ring).
- [ ] The `prefers-reduced-motion` CSS media query is honored in the frontend stylesheet.

---

## Zoom and Text Scaling

- [ ] With the OS or browser text size increased to 200%, the application remains usable — text does not overflow its containers to the point of being unreadable, and controls remain accessible.
- [ ] With OS display scaling set to 150%, the layout does not break.
- [ ] No critical information is clipped or hidden when the user zooms in.

---

## Error Message Accessibility

- [ ] Error messages are associated with their triggering input via `aria-describedby` where applicable.
- [ ] Error states on inputs set `aria-invalid="true"` so screen readers announce the field as invalid.
- [ ] Error summaries at the top of a form (if present) receive focus or are announced via `role="alert"` so the user is immediately aware of them.
- [ ] Errors are not communicated only through placeholder text changes or color changes.

---

## Test Tools

| Tool | Purpose |
|------|---------|
| axe-core (via browser DevTools) | Automated accessibility rule checking against WCAG 2.1 AA |
| NVDA (Windows) | Manual screen reader testing on Windows |
| VoiceOver (macOS) | Manual screen reader testing on macOS |
| Colour Contrast Analyser | Manual contrast ratio verification for custom UI elements |
| OS "Reduce Motion" setting | Verifying animation suppression |

**Using axe-core:**

1. Open the application in a Chromium-based WebView or browser-based dev session.
2. Open DevTools and navigate to the axe DevTools extension panel.
3. Run a full-page scan.
4. All "Critical" and "Serious" violations must be resolved before release. "Moderate" and "Minor" violations should be filed as issues.
