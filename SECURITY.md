# Security policy

## Supported versions

Only the latest release is supported. Security fixes are issued as a new
release rather than backported.

| Version | Supported |
| --- | --- |
| 2.x | yes |
| < 2.0 | no |

## Reporting a vulnerability

Please do not open a public issue for a security problem.

Use GitHub's private [Security Advisories](https://github.com/zhafribs/lifting-plan-calculator/security/advisories/new)
("Report a vulnerability") to send the details. Include:

- what the issue is and its impact,
- how to reproduce it,
- the version and platform affected.

You will get an acknowledgement as soon as the report is seen. If the issue is
confirmed, a fix and a release will follow, and you will be credited unless you
ask otherwise.

## Scope

The application runs entirely on the user's machine. The threat surface is
therefore small: reading `.xlsx` load charts, writing the saved HTML report, and
the embedded web view (the Tauri shell). Reports about the bundled
GTK/WebKitGTK stack should be raised upstream, though a heads-up here is
welcome.

### The PDF export downloads nothing

*Save as PDF* prints the report with the app's own WebKitGTK view, in-process.
This is an invariant, not an implementation detail: the AppImage carries no
browser, downloads no renderer, and executes no program it fetched. A change
that fetches or runs a renderer at export time is a security regression and
should be treated as one.

### Network access

The application does not collect or send telemetry, and it sends nothing about
a plan. Two features do contact the network, and only when the operator uses
them:

- **Update check** asks `api.github.com` for the latest release when the app
  starts. It sends no identifying information, and it can be ignored entirely
  by not installing an update.
- **Update install** downloads the release installer the check offered, from
  `github.com`, and runs it — only after the operator asks for it.

The PDF export, the calculator, the chart import and the report are all
offline.

