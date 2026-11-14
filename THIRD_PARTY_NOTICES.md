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

## Bundled with the AppImage, fetched at build time

| Component | Version | License | Purpose | Full text |
| --- | --- | --- | --- | --- |
| [Chrome for Testing](https://googlechromelabs.github.io/chrome-for-testing/) (headless shell) | 157.0.8103.78 | BSD-3-Clause (Chromium) and the third-party licenses it lists | Rendering the PDF export without a browser installed on the target machine | `pdf-renderer/chrome-headless-shell-linux64/LICENSE.headless_shell` inside the payload |

The renderer is downloaded by `scripts/fetch-pdf-renderer.sh` from the Chrome
for Testing archive, with the version and the archive checksum pinned. It is
not stored in this repository. Its license text ships inside the bundle beside
the binary; keep it there when redistributing the AppImage.
