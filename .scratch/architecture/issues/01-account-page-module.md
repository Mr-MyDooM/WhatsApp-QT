# 01: Account page module

**What to build:** one pure module, `src/page.rs`, owns everything the app sends to an account page (see GLOSSARY.md): theme CSS with the WhatsApp DOM selectors, the init script, and the live update. The core sends one page config for every page setting. Today a setting reaches the page by four paths (init globals, `on_page_load`, the `ApplyTheme`/`ApplyZoom` arms, and `eval` loops in `set_spellcheck` and `set_chat_list_width`), and the JS names and width limits are spelled by hand on both sides.

**Blocked by:** None (can start immediately).

**Status:** resolved

## Decisions

- `src/page.rs` has no Tauri types. It turns a `PageConfig` into the init script and the live update JS. `theme_css` and its selectors move out of `core.rs` into it.
- `PageConfig` holds theme CSS, zoom, spellcheck, chat list width, and the chat list min and max width.
- One Effect: `ApplyPageConfig { target, config }`, where target is all accounts or one account id. It replaces `ApplyTheme` (page part) and `ApplyZoom`.
- `run_effects` does one `set_zoom` and one `eval` per account page.
- New Effect `SetWindowScheme` takes the main window color scheme part of `ApplyTheme`.
- `Event::PageLoaded` returns `ApplyPageConfig` for that account. `on_page_load` stops reading the core.
- JS: one global `window.__PAIRLINE_CONFIG__` at init, one entry `window.__pairlineApply(config)` for live updates. Each settings shim registers a handler. The global names are written only in `page.rs` and one small bootstrap shim.
- A new theme shim replaces `inject_css`.
- `chat_list_shim.js` reads min and max from the config and drops its copy. `CHAT_LIST_MIN_WIDTH` and `CHAT_LIST_MAX_WIDTH` in `core.rs` stay the only source.
- The avatar and notification shims do not change.
- `set_chat_list_width` keeps its account-page caller check and still updates all account pages, the one that sent the change included.

## Done when

- [ ] Theme, zoom, spellcheck and chat list width each reach every account page through `ApplyPageConfig` only.
- [ ] Core tests: each page setting change returns `ApplyPageConfig` for all accounts with the new value; `PageLoaded` returns it for one account.
- [ ] `page.rs` tests: the init script and the live update JS hold the config as JSON, and the theme CSS covers each theme (move `theme_css_covers_each_theme`).
- [ ] No settings JS is built with `format!` in `main.rs`. `inject_css`, the two `eval` loops and the init globals in `add_account_webview` are gone.
- [ ] Manual check in the bundled app: change theme, zoom, spellcheck and drag the chat list; open a second account and reload a page; each setting holds.

## Answer

Done on `refactor/architecture`. Commit 16fb1a0. Review fix 09ef0c0: the theme and spellcheck shims skip work when the value is unchanged. The four checks exit 0 at 831d0e4 (145 tests). Not done: the manual check in the bundled `.app`.
