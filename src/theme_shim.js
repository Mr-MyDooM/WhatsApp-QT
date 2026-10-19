(function () {
  // Shows the theme CSS from the page config. Empty CSS removes the theme. The page load
  // sends the config again, so a head that WhatsApp replaced gets the style back.
  var ID = "app-theme-style";
  window.__pairlineOnConfig(function (config) {
    var old = document.getElementById(ID);
    if (old) old.remove();
    if (!config.theme_css) return;
    var style = document.createElement("style");
    style.id = ID;
    style.textContent = config.theme_css;
    (document.head || document.documentElement).appendChild(style);
  });
})();
