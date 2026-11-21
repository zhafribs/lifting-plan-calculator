//! The report's stylesheet, kept apart from the code that assembles the page.
//!
//! It is one string on purpose: the same CSS is handed to the page view, to the
//! PDF export and to the browser fallback, so a rule added here reaches all
//! three. Rules that need the page margins come from `PageMargins` at print
//! time, not from this file.

pub const REPORT_CSS: &str = r#"
<style>
@page {
    @bottom-left { content: var(--plan-reference, "No reference"); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
    @bottom-center { content: var(--footer-center, "Prepared with Lifting Plan Calculator by Zhafri Syazwi — zhafribs@gmail.com"); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
    @bottom-right { content: counter(page) " / " counter(pages); font-size: 6pt;
        color: #555; font-family: "Liberation Serif", "DejaVu Serif", serif; }
}
.report {
    color: #151515;
    background: #fff;
    font-family: "Liberation Serif", "DejaVu Serif", serif;
    font-size: 7pt;
    line-height: 1.2;
}
.report h1 { font-size: 9pt; margin: 0 0 4pt; color: #111; text-align: center; }
.report h2 { font-size: 6pt; margin: 11pt 0 3pt; padding: 2pt 4pt;
    background: #e8edf3; border-bottom: 1px solid #9aa9ba; }
.report h3 { font-size: 6pt; margin: 7pt 0 1pt; }
.report p { margin: 1.5pt 0; }
.report .meta { color: #555; font-size: 5.5pt; margin-bottom: 5pt; }
.report table { border-collapse: collapse; width: 100%; margin: 3pt 0 6pt; }
.report table.kv { table-layout: fixed; }
.report table.kv th { width: 38%; }
.report table.kv td { width: 62%; }
.report table.kv.light { border-collapse: collapse; margin: 2pt 0 5pt; }
.report table.kv.light th { border: 0; border-bottom: 0.5pt solid #e4eaf1;
    background: none; padding: 2.5pt 8pt 2.5pt 0; font-size: 5.5pt;
    font-weight: normal; letter-spacing: 0.3pt; text-transform: uppercase;
    color: #6d7a88; vertical-align: top; }
.report table.kv.light td { border: 0; border-bottom: 0.5pt solid #e4eaf1;
    padding: 2.5pt 0; font-size: 7.5pt; color: #1b2733; vertical-align: top; }
.report table.kv.light tr:last-child th,
.report table.kv.light tr:last-child td { border-bottom: 0; }
.report td, .report th { border: 1px solid #7c8998; padding: 1pt 2pt;
    vertical-align: top; font-size: 5.5pt; }
.report th { background: #f1f4f7; text-align: left; font-weight: bold; }
.report .equation { margin: 2pt 0 3pt 12pt; font-family: "DejaVu Serif", serif;
    font-size: 5pt; }
.report .equation .fallback { font-family: "Liberation Serif", serif; }
.report .note { border: 1px solid #9aa9ba; background: #f7f7f7; padding: 5pt 7pt;
    margin-top: 5pt; }
.report .critical { border: 2px solid #a33; padding: 5pt 7pt; margin: 5pt 0; }
.report .pass { color: #176b3a; font-weight: bold; }
.report .fail { color: #9b1c1c; font-weight: bold; }
.report .small { font-size: 7pt; } .report .note { font-size: 7pt; } .report .status { font-size: 7pt; } .report .meta { font-size: 7pt; } .report p { font-size: 7pt; } .report td, .report th { font-size: 7pt; } .report .equation { font-size: 6.5pt; } .report h1 { font-size: 12pt; } .report h2 { font-size: 9pt; } .report h3 { font-size: 8pt; }
/* The plan's head: a light card that reads top-down — what the plan is, the
   figures that decide it, then the verdict. No heavy grid; the values carry
   the weight and the labels stay quiet. */
.report { -webkit-print-color-adjust: exact; print-color-adjust: exact; }
.report .plan-head { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 5pt; margin: 0 0 6pt; }
.report .plan-title-block { margin: 0 0 5pt; }
.report .plan-title-block h1 { font-size: 16pt; line-height: 1; margin: 0;
    text-align: left; letter-spacing: -0.3pt; color: #101820; }
.report .plan-crane { font-size: 10pt; font-weight: bold; color: #1b2733; }
.report .plan-details { margin-top: 4pt; }
.report .plan-head h2 { margin: 3pt 0 2pt; }
.report table.plan-meta { width: 100%; border-collapse: collapse;
    table-layout: fixed; margin: 0; }
.report table.plan-meta col.ref { width: 30%; }
.report table.plan-meta col.issued { width: 22%; }
.report table.plan-meta col.pb,
.report table.plan-meta col.cb,
.report table.plan-meta col.ab { width: 16%; }
.report table.plan-meta td { border: 0; padding: 0 5pt 0 0; vertical-align: top; }
.report .plan-meta .k { display: block; font-size: 5.5pt; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; }
.report .plan-meta .v { display: block; font-size: 7.5pt; color: #1b2733;
    border-bottom: 0.5pt solid #ccd6e0; padding: 1pt 0 1.5pt; min-height: 8pt; }
.report table.metrics { width: 100%; border-collapse: separate;
    border-spacing: 3pt 2pt; margin: 2pt -3pt 0; table-layout: fixed; }
.report table.metrics td { border: 0; padding: 3pt 5pt; background: #f2f6fa;
    border-radius: 3pt; vertical-align: top; }
.report .input-card { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 6pt; margin: 0 0 6pt; }
.report .input-card h2 { margin: 1pt 0 4pt; }
.report .section-card { border: 1px solid #d9e1e9; border-radius: 5pt;
    background: #fbfcfe; padding: 5pt 9pt 6pt; margin: 0 0 6pt; }
.report .section-card h2 { margin: 1pt 0 4pt; }
.report table.input-grid { width: 100%; border-collapse: separate;
    border-spacing: 3pt 2pt; margin: 0 -3pt; table-layout: fixed; }
.report table.input-grid td { border: 0; padding: 3.5pt 5pt; background: #f2f6fa;
    border-radius: 3pt; vertical-align: top; }
.report table.input-grid .input-v { font-size: 8pt; }
.report .line-list-title { font-size: 6pt; font-weight: bold;
    letter-spacing: 0.3pt; text-transform: uppercase; color: #6d7a88;
    margin: 6pt 0 1.5pt; }
.report table.line-list { width: 100%; border-collapse: collapse;
    table-layout: auto; margin: 0 0 3pt; font-size: 7pt; }
.report table.line-list th { border: 0; border-bottom: 0.5pt solid #ccd6e0;
    padding: 1.5pt 3pt; text-align: left; font-size: 5.5pt;
    letter-spacing: 0.2pt; text-transform: uppercase; color: #6d7a88;
    font-weight: normal; }
.report table.line-list td { border: 0; border-bottom: 0.5pt solid #e4eaf1;
    padding: 2pt 3pt; color: #1b2733; vertical-align: top; }
.report table.line-list .n { text-align: right; white-space: nowrap; }
.report table.line-list tr:last-child td { border-bottom: 0; }
.report table.line-list tr.group td { border-bottom: 0; padding: 3pt 3pt 1pt;
    font-size: 5.5pt; font-weight: bold; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; }
.report .pad-square-line { display: flex; align-items: baseline; gap: 5pt;
    margin: 3pt 0 2pt; }
.report .pad-square-k { font-size: 6.5pt; font-weight: bold;
    letter-spacing: 0.3pt; text-transform: uppercase; color: #6d7a88;
    white-space: nowrap; }
.report .pad-square-eq .katex { font-size: 1em; }
.report .working-wrap { padding: 0; margin-top: 6pt; }
.report table.working { width: 100%; border-collapse: collapse; margin: 4pt 0 4pt;
    /* Auto, not fixed: the divider sits at the true centre when both sides fit,
       and moves to fit whichever side needs more room when they do not. */
    table-layout: auto; }
.report table.working td { border: 0; padding: 0; vertical-align: top;
    break-inside: avoid; page-break-inside: avoid; }
/* A factor's label and formula are one row; never print them on two pages. */
.report table.working tr { break-inside: avoid; page-break-inside: avoid; }
/* The choke reduction is two tables side by side, without a frame of its own. */
.report table.choke-split { margin: 2pt 0; }
.report table.choke-split > tbody > tr > td { border: 0; padding: 0 5pt 0 0;
    vertical-align: top; }
.report table.choke-split > tbody > tr > td:last-child { padding-right: 0; }
/* Notes read as prose: left-aligned, the choke column wider. */
.report table.working.notes .work-label { text-align: left; }
.report table.working.notes td { text-align: left; }
.report table.working.notes p { margin: 1pt 0; }
/* Rows of factors need air between them. */
.report table.working tr + tr td { padding-top: 5pt; }
/* A heading is never the last thing on a page. */
/* Keep-with-next, stated on both sides: a heading never ends a page, and the
   block it introduces never starts one. This applies to every heading in the
   report — sections live in more than one container class (`section-card`,
   `input-card`), and scoping the rule to one of them is what left "Lift Input
   Parameters" alone at the foot of a page. */
.report h2, .report h3 { break-after: avoid; page-break-after: avoid;
    break-inside: avoid; }
.report h2 + *, .report h3 + * { break-before: avoid; page-break-before: avoid; }
/* The operator's own page breaks: a section that starts on a fresh page. */
.report .new-page { break-before: page; page-break-before: always; }
/* No width hints: the columns take the room their content needs, so the
   divider sits at the true centre when both sides fit and moves to fit the
   wider side when they do not. */
.report table.working td:first-child { padding-right: 8pt; }
.report table.working td:last-child { padding-left: 8pt;
    border-left: 0.5pt solid #e4eaf1; }
.report .work-label { font-size: 7pt; font-weight: bold; color: #1b2733;
    text-align: center; padding-bottom: 3pt; margin-bottom: 5pt;
    border-bottom: 0.5pt solid #d9e1e9; }
.report table.working .equation { margin: 0; font-size: 6pt; text-align: center; }
.report table.working .equation .katex-display { text-align: center; margin: 0; }
.report table.working .equation .katex-display > .katex { text-align: center; }
.report .work-empty { font-size: 6pt; color: #6d7a88; }
.report .working-diagram { margin-top: 6pt; text-align: center; }
.report .diagram-title { font-size: 6pt; font-weight: bold; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; margin-bottom: 2pt; }
/* The diagram keeps the height it always had; the capture is what makes it
   sharp (3x, with its palette turned over, so no CSS filter softens it). */
.report .working-diagram img { display: block; margin: 0 auto;
    width: auto; height: auto; max-width: 100%; max-height: 78mm;
    border: 0.5pt solid #d9e1e9; border-radius: 3pt; background: #fff; }
.report .metric-k { display: block; font-size: 5.5pt; letter-spacing: 0.3pt;
    text-transform: uppercase; color: #6d7a88; }
.report .metric-v { display: block; font-size: 10pt; font-weight: bold;
    line-height: 1.1; color: #0f1720; margin-top: 0.5pt; }
.report .metric-line { display: flex; align-items: baseline; gap: 4pt; }
.report .metric-line .metric-v { margin: 0; }
.report .metric-line .metric-band { margin: 0; }
.report .metric-n { display: block; font-size: 5.5pt; color: #5d6b7a;
    margin-top: 1pt; line-height: 1.2; }
.report .metric-head { display: flex; align-items: baseline; gap: 3pt;
    flex-wrap: wrap; }
.report .metric-aside { font-size: 5.5pt; letter-spacing: 0;
    text-transform: none; color: #6d7a88; font-weight: normal; }
.report .metric-band { display: block; font-size: 8pt; font-weight: bold;
    letter-spacing: 0.3pt; line-height: 1.1; margin-top: 1pt; }
.report .metric-sub { display: block; font-size: 6pt; color: #5d6b7a;
    line-height: 1.2; }
.report .metric-band.within { color: #176b3a; }
.report .metric-band.caution { color: #8a5517; }
.report .metric-band.over { color: #9b1c1c; }
.report .metric-band.neutral { color: #4a5560; }
</style>
"#;
