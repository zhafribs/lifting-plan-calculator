# PDF renderer payload

`scripts/fetch-pdf-renderer.sh` unpacks Chrome for Testing's headless shell
here (~128 MB download, pinned version and checksum). The AppImage build
carries it so the PDF export needs no browser on the target machine; this
placeholder keeps the bundle's resource glob valid on a machine that has not
fetched it.

The binary is not in version control.
