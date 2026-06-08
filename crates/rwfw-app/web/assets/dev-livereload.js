// Dev-only live reload. Loaded by base.html.j2 only when `is_development`.
// Subscribes to the server's file watcher over SSE and either hot-swaps the
// stylesheet (CSS change — no reload, keeps scroll/form state) or reloads the
// page (template/asset change). Hand-written, npm-free, no build step.
(function () {
  "use strict";

  var everConnected = false;

  function connect() {
    var source = new EventSource("/__rwfw/livereload");

    source.onopen = function () {
      if (everConnected) {
        // Reconnected after the dev server restarted — reload to catch up.
        window.location.reload();
        return;
      }
      everConnected = true;
    };

    source.onmessage = function (event) {
      if (event.data === "css") {
        swapStylesheets();
      } else {
        reloadPage();
      }
    };

    // EventSource reconnects automatically on error; nothing to do here.
  }

  function swapStylesheets() {
    var links = document.querySelectorAll('link[rel="stylesheet"]');
    links.forEach(function (link) {
      var href = link.getAttribute("href");
      if (!href || href.indexOf("/assets/") === -1) {
        return; // only our own stylesheets
      }
      var fresh = link.cloneNode();
      fresh.setAttribute("href", href.split("?")[0] + "?v=" + Date.now());
      // Drop the stale sheet only once the fresh one has loaded — no FOUC.
      fresh.addEventListener("load", function () {
        link.remove();
      });
      link.parentNode.insertBefore(fresh, link.nextSibling);
    });
  }

  function reloadPage() {
    if (window.Turbo && typeof window.Turbo.visit === "function") {
      // Turbo 8 morph: replaces the body, preserving scroll where possible.
      window.Turbo.visit(window.location.href, { action: "replace" });
    } else {
      window.location.reload();
    }
  }

  connect();
})();
