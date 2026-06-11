import "@testing-library/jest-dom";
import { configureAxe } from "jest-axe";

configureAxe({
  rules: {
    // Color contrast is checked manually per design token; skip in automated smoke tests
    "color-contrast": { enabled: false },
  },
});
