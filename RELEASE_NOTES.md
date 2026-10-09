# Release Notes

## 2.2.1

### Fixed

- **No more "ResizeObserver loop completed with undelivered notifications."
  banner on the Graph tab.** The working-range diagram resized its canvas bitmap
  inside the ResizeObserver delivery cycle, which WebKitGTK reports as an error;
  the redraw is now deferred to the next animation frame. The global error
  handler additionally treats that specific recoverable message as a warning —
  it is logged, never shown as a startup-error banner.

### Artifacts

- `Lifting-Plan-Calculator-2.2.1-linux-x86_64.AppImage` (plus `.zsync`) —
  Ubuntu 22.04+, Fedora 36+ and Arch (glibc 2.35 floor)
- `Lifting-Plan-Calculator-2.2.1-windows-x86_64.msi` — Windows 10/11, x64
- `Lifting-Plan-Calculator-2.2.1-windows-x86_64-setup.exe` — NSIS installer
- `SHA256SUMS.txt` — verify any asset with `sha256sum -c SHA256SUMS.txt`