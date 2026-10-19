---
status: accepted
---

# React, TypeScript and shadcn/ui for the app chrome, following the macOS HIG

The app chrome (sidebar, settings, rename window, error panel) moves from hand-written HTML, CSS and JavaScript to React with TypeScript, built by Vite, with Tailwind and shadcn/ui components. The look follows Apple's Human Interface Guidelines: translucent sidebar, system font, native light and dark appearance, a Settings window with toolbar panes. This replaces the spec's "plain static HTML/CSS/JS, no npm, no framework" rule. Reason: the hand-written chrome stayed at the bare minimum, and the user wants the industry standard for each part of the app.

## Considered Options

- **React + TypeScript + Vite + shadcn/ui** (chosen): the most common Tauri frontend stack. shadcn/ui is built on Radix, so keyboard and screen-reader support come with the components.
- **Svelte + TypeScript + Vite**: smaller bundle, but fewer ready components, so more hand styling.
- **Plain HTML/CSS/JS with a hand-written design system**: no npm, but every component and its accessibility is written by hand.
- **Custom brand look** (like Discord or Beeper) instead of the macOS HIG: rejected because macOS is the first platform.

## Consequences

- The build needs Node and pnpm. `cargo tauri build` runs the frontend build first, and `cargo run` needs the built frontend or the Vite dev server.
- Pin exact package versions. Commit the lock file.
- Only local chrome uses React. The `web.whatsapp.com` webviews are unchanged and still get exactly one IPC command. The strict CSP stays.
