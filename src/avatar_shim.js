(function () {
  if (window.__TAURI_INTERNALS__ === undefined) return;
  // The user's OWN avatar: the profile picture in the header of the left pane. Chat list
  // and chat header pictures belong to other people and must never match.
  // WhatsApp can change its page at any time. When nothing matches, this does nothing and
  // the app shows colored initials.
  var SELECTOR = "#side > header img, header button[aria-label='Profile'] img";
  var SIZE = 96;
  var seen = "";
  function send(img) {
    var probe = new Image();
    probe.crossOrigin = "anonymous";
    probe.onload = function () {
      try {
        var side = Math.min(probe.naturalWidth, probe.naturalHeight);
        if (!side) return;
        var canvas = document.createElement("canvas");
        canvas.width = SIZE;
        canvas.height = SIZE;
        canvas
          .getContext("2d")
          .drawImage(
            probe,
            (probe.naturalWidth - side) / 2,
            (probe.naturalHeight - side) / 2,
            side,
            side,
            0,
            0,
            SIZE,
            SIZE,
          );
        window.__TAURI_INTERNALS__
          .invoke("profile_picture", { dataUrl: canvas.toDataURL("image/jpeg", 0.8) })
          .catch(function () {});
      } catch (e) {
        // A tainted canvas throws here. Initials stay.
      }
    };
    probe.src = img.src;
  }
  setInterval(function () {
    var img = document.querySelector(SELECTOR);
    var src = img && img.complete && img.naturalWidth ? img.src : "";
    if (!src || src === seen) return;
    seen = src;
    send(img);
  }, 5000);
})();
