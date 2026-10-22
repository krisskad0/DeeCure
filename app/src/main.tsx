// SPDX-License-Identifier: Apache-2.0

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import App from "./App";
import "@fontsource-variable/ubuntu-sans";
import "@fontsource-variable/ubuntu-sans-mono";
import "./tokens.css";
import "./styles/base.css";
import "./styles/components.css";
import "./styles/shell.css";

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <App />
    </StrictMode>,
  );
}
