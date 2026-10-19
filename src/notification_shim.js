(function () {
  if (window.__TAURI_INTERNALS__ === undefined) return;
  function send(title, options) {
    var body = options && options.body !== undefined ? String(options.body) : "";
    window.__TAURI_INTERNALS__
      .invoke("notify", { title: String(title), body: body })
      .catch(function () {});
  }
  function Notification(title, options) {
    send(title, options);
  }
  Notification.permission = "granted";
  Notification.requestPermission = function (callback) {
    if (typeof callback === "function") callback("granted");
    return Promise.resolve("granted");
  };
  Notification.prototype.close = function () {};
  Notification.prototype.addEventListener = function () {};
  Notification.prototype.removeEventListener = function () {};
  window.Notification = Notification;
})();
