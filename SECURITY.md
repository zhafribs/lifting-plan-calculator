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

The application runs entirely on the user's machine and does not phone home or
store data between launches. The threat surface is therefore small: reading
`.xlsx` load charts, writing the saved HTML report, and the embedded web view
(the Tauri shell). Reports about the bundled GTK/WebKitGTK stack should be
raised upstream, though a heads-up here is welcome.
