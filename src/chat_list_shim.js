(function () {
  if (window.__TAURI_INTERNALS__ === undefined) return;
  // Lets the user drag the right edge of WhatsApp's chat list (#side) to resize it. A double
  // click on the edge gives back WhatsApp's own width.
  // WhatsApp can change its page at any time. When #side is missing, this does nothing.
  // The limits come from the page config. null width means WhatsApp's own width.
  var MIN;
  var MAX;
  var HANDLE = "data-pairline-resize";
  var COLUMN = "data-pairline-column";
  var width = null;

  // The outermost box that is as wide as the chat list: WhatsApp sizes that box.
  function pane() {
    var el = document.getElementById("side");
    while (
      el &&
      el.parentElement &&
      el.parentElement.id !== "app" &&
      el.parentElement.getBoundingClientRect().width - el.getBoundingClientRect().width < 2
    ) {
      el = el.parentElement;
    }
    return el;
  }

  // In a column flex box, flex-basis sets the height, which cuts off the Status and Channels
  // lists. So set flex only when the parent lays out in a row.
  function isRow(el) {
    var s = el.parentElement && getComputedStyle(el.parentElement);
    return !!s && s.display.indexOf("flex") !== -1 && s.flexDirection.indexOf("row") === 0;
  }

  function size(el) {
    el.style.flex = width && isRow(el) ? "0 0 " + width + "px" : "";
    el.style.width = width ? width + "px" : "";
    el.style.maxWidth = width ? "none" : "";
  }

  // A layer above the page (absolute, beside the pane) holds a column as wide as WhatsApp's own
  // chat list. The next column has a left border, which shows as a line at the old edge. Give that
  // column the same width. Marked with COLUMN so a reset clears only what this script set.
  function columns(p) {
    var left = p.getBoundingClientRect().left;
    var found = Array.prototype.slice.call(document.querySelectorAll("[" + COLUMN + "]"));
    Array.prototype.forEach.call(p.parentElement.children, function (layer) {
      if (!width || layer === p || getComputedStyle(layer).position !== "absolute") return;
      var layerWidth = layer.getBoundingClientRect().width;
      layer.querySelectorAll("div").forEach(function (el) {
        var r = el.getBoundingClientRect();
        if (Math.abs(r.left - left) > 1 || r.width >= layerWidth - 2 || r.height < innerHeight / 2) return;
        if (found.indexOf(el) === -1) found.push(el);
      });
    });
    found.forEach(function (el) {
      el.setAttribute(COLUMN, "");
      size(el);
    });
  }

  function apply(p) {
    size(p);
    columns(p);
  }

  function save() {
    window.__TAURI_INTERNALS__.invoke("set_chat_list_width", { width: width }).catch(function () {});
  }

  function attach() {
    var p = pane();
    if (!p) return;
    apply(p);
    if (p.querySelector("[" + HANDLE + "]")) return;
    if (getComputedStyle(p).position === "static") p.style.position = "relative";
    var handle = document.createElement("div");
    handle.setAttribute(HANDLE, "");
    handle.style.cssText =
      "position:absolute;top:0;right:0;bottom:0;width:6px;cursor:col-resize;z-index:1000;";
    handle.addEventListener("mousedown", function (down) {
      down.preventDefault();
      var startX = down.clientX;
      var startWidth = p.getBoundingClientRect().width;
      function move(e) {
        width = Math.round(Math.min(MAX, Math.max(MIN, startWidth + e.clientX - startX)));
        apply(p);
      }
      function up() {
        document.removeEventListener("mousemove", move);
        document.removeEventListener("mouseup", up);
        save();
      }
      document.addEventListener("mousemove", move);
      document.addEventListener("mouseup", up);
    });
    handle.addEventListener("dblclick", function () {
      width = null;
      apply(p);
      save();
    });
    p.appendChild(handle);
  }

  window.__pairlineOnConfig(function (config) {
    MIN = config.chat_list_min;
    MAX = config.chat_list_max;
    width = config.chat_list_width;
    attach();
  });
  // WhatsApp builds and rebuilds its page after load, so look again every second.
  setInterval(attach, 1000);
})();
