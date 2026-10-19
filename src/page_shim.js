(function () {
  // The Rust side sets window.__PAIRLINE_CONFIG__ before this script runs, and calls
  // window.__pairlineApply(config) when a setting changes. Each settings shim registers
  // a handler with window.__pairlineOnConfig. A handler runs at once if a config exists.
  var handlers = [];
  window.__pairlineOnConfig = function (handler) {
    handlers.push(handler);
    if (window.__PAIRLINE_CONFIG__) handler(window.__PAIRLINE_CONFIG__);
  };
  window.__pairlineApply = function (config) {
    window.__PAIRLINE_CONFIG__ = config;
    handlers.forEach(function (handler) {
      handler(config);
    });
  };
})();
