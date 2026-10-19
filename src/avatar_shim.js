(function () {
  if (window.__TAURI_INTERNALS__ === undefined) return;
  // The user's OWN avatar: the last picture in the left rail header (older pages: the
  // header of #side). The rail also holds WhatsApp's own icons, such as Meta AI, which come
  // from STATIC_HOST and are skipped. The own picture can load after them.
  // Chat list and chat header pictures belong to other people and must never match.
  // WhatsApp can change its page at any time. When nothing matches, this does nothing and
  // the app shows colored initials.
  var STATIC_HOST = "://static.whatsapp.net/";
  // Longer than MIN_CALL_GAP_SECS in avatar.rs. A faster call is dropped there, and this
  // script does not send the same picture again.
  var POLL_MS = 11000;
  function findOwnAvatar() {
    var old = document.querySelector("#side > header img");
    if (old) return old;
    var headers = document.querySelectorAll("header");
    for (var i = 0; i < headers.length; i++) {
      if (headers[i].closest("#main, #side")) continue;
      var imgs = Array.prototype.filter.call(headers[i].querySelectorAll("img"), function (img) {
        return img.src.indexOf(STATIC_HOST) === -1;
      });
      return imgs.length ? imgs[imgs.length - 1] : null;
    }
    return null;
  }
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
    var img = findOwnAvatar();
    var src = img && img.complete && img.naturalWidth ? img.src : "";
    if (!src || src === seen) return;
    seen = src;
    send(img);
  }, POLL_MS);
})();
