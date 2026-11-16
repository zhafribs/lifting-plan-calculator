// Typesets the report's equations once the document is parsed.
//
// A file rather than an inline script: the app runs under a Content-Security
// Policy that allows scripts only from its own origin, so an inline pass would
// be blocked. When the report is shown in the preview frame it also reports how
// many equations it set, which is the one thing the window cannot show.
document.addEventListener("DOMContentLoaded", function () {
  var equations = 0;
  try {
    renderMathInElement(document.body, {
      delimiters: [
        { left: "\\(", right: "\\)", display: false },
        { left: "\\[", right: "\\]", display: true },
      ],
      throwOnError: false,
      strict: false,
    });
    equations = document.querySelectorAll(".katex").length;
  } catch (error) {
    document.body.setAttribute("data-katex-error", error.message);
  }
  if (window.parent && window.parent !== window) {
    var failure = document.body.getAttribute("data-katex-error");
    window.parent.postMessage(
      { lpcReport: failure ? "error" : "ready", equations: equations, error: failure || "" },
      "*",
    );
  }
});
