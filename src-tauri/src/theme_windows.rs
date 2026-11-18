//! Windows-only theme helpers.
//!
//! The Windows build follows the system accent colour. DWM exposes it as an
//! `0xAARRGGBB` value; the UI wants plain `#rrggbb`, so this module does the
//! conversion and keeps the FFI in one place.

use windows::Win32::Graphics::Dwm::DwmGetColorizationColor;

/// The current Windows accent colour as `#rrggbb`, or `None` when DWM cannot
/// answer (composition disabled, a locked-down session, or an API failure).
pub fn system_accent() -> Option<String> {
    let mut argb = 0u32;
    let mut opaque = 0;
    // SAFETY: both pointers refer to live, correctly typed locals for the
    // duration of the call.
    unsafe { DwmGetColorizationColor(&mut argb, &mut opaque) }.ok()?;
    // The alpha byte is not part of the colour the UI paints with.
    Some(format!("#{:06x}", argb & 0x00ff_ffff))
}
