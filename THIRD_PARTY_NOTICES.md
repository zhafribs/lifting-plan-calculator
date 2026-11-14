# Third-party notices

The Lifting Plan Calculator is licensed under the GNU General Public License,
version 3 or later (see [LICENSE](LICENSE)). It bundles and links against the
following third-party components. Each remains under its own license, and the
notices below are provided to satisfy those licenses.

## Bundled in the repository

| Component | Version | License | Purpose | Full text |
| --- | --- | --- | --- | --- |
| [KaTeX](https://katex.org/) | see `ui/vendor/katex/` | MIT | Typesetting the formulas in the HTML report | [licenses/KaTeX-MIT.txt](licenses/KaTeX-MIT.txt) |
| [Inter](https://rsms.me/inter/) | variable | SIL Open Font License 1.1 | UI and report typeface | [licenses/Inter-OFL-1.1.txt](licenses/Inter-OFL-1.1.txt) |
| Rust crates | see `src-tauri/Cargo.lock` | MIT / Apache-2.0 / BSD / ISC / MPL-2.0 / Zlib / Unicode-3.0 and other permissive licenses | Calculation engine, GUI toolkit, spreadsheets, file dialogs | upstream `LICENSE` files |

The complete, transitive Rust dependency set is pinned in
`src-tauri/Cargo.lock`. Every crate in it is distributed under a permissive or
GPL-3.0-compatible license; none is proprietary or GPL-incompatible.

## Bundled inside the AppImage

The AppImage produced by the build scripts packages the application together
with the GTK 3 and WebKitGTK 4.1 runtime stack it needs. Those libraries are
dynamic libraries under the LGPL (GTK, WebKitGTK) and other free licenses, and
are included unmodified. Their source is available from the upstream projects
and from the Ubuntu 22.04 archive the portable build uses.

If you redistribute the AppImage, keep this file and `LICENSE` with it, and
comply with the LGPL for the bundled GTK/WebKitGTK libraries (in particular,
your recipients' ability to relink those libraries).

## Runtime components

The AppImage carries no browser: the PDF export prints the report with the
WebKitGTK view the application already runs on, and the footer is drawn onto
the finished pages with [lopdf](https://crates.io/crates/lopdf) (MIT licence,
statically linked). Windows uses the Microsoft Edge that ships with the
operating system.

The footer's typeface is embedded in the PDF export and in this application:

| Component | Version | License | Purpose | Full text |
| --- | --- | --- | --- | --- |
| [Liberation Serif](https://github.com/liberationfonts/liberation-fonts) | 2.x | SIL Open Font License 1.1 | The footer text on every exported page | `src-tauri/assets/fonts/LICENSE-LiberationSerif.txt` |
| [pdf.js](https://github.com/mozilla/pdf.js) (`pdfjs-dist`) | 3.11.174 | Apache-2.0 | Drawing the report's pages in Summary | `licenses/pdfjs-Apache-2.0.txt` and `ui/vendor/pdfjs/LICENSE` |

The two pdf.js files are committed unmodified and pinned by SHA-256 in
`ui/vendor/pdfjs/CHECKSUMS.txt`; `packaging/check-vendored.sh` fails the build if
they change. Nothing is fetched at build or run time.
