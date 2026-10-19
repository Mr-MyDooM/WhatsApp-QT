(function () {
  if (window.__TAURI_INTERNALS__ === undefined) return;
  // Turns spellcheck off in WhatsApp's text boxes when the user switches it off in Settings.
  // Marked with MARK so switching it on again clears only what this script set.
  var MARK = "data-pairline-spellcheck";
  var enabled = true;

  function apply() {
    if (enabled) {
      document.querySelectorAll("[" + MARK + "]").forEach(function (el) {
        el.removeAttribute("spellcheck");
        el.removeAttribute(MARK);
      });
      return;
    }
    document.querySelectorAll("[contenteditable], input, textarea").forEach(function (el) {
      if (el.hasAttribute(MARK)) return;
      el.setAttribute("spellcheck", "false");
      el.setAttribute(MARK, "");
    });
  }

  window.__pairlineOnConfig(function (config) {
    enabled = config.spellcheck;
    apply();
  });
  // WhatsApp makes new text boxes as the user moves between chats, so look again every second.
  setInterval(apply, 1000);
})();
