import type { ReactNode } from "react";
import { createRoot } from "react-dom/client";
import "../index.css";

// Only macOS draws the window vibrancy behind the sidebar: see `main.rs`.
if (navigator.userAgent.includes("Macintosh")) {
  document.documentElement.classList.add("vibrant");
}

export function mount(app: ReactNode) {
  createRoot(document.getElementById("root")!).render(app);
}
