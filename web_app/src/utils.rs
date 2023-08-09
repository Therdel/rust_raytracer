use std::slice;

use crate::color::ColorRgbaU8;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

pub fn canvas_from_raw_mut(canvas_u8: &mut [u8]) -> &mut [ColorRgbaU8] {
    let canvas_raw_color = canvas_u8.as_mut_ptr() as *mut ColorRgbaU8;
    unsafe {
        slice::from_raw_parts_mut(canvas_raw_color, canvas_u8.len() / std::mem::size_of::<ColorRgbaU8>())
    }
}
