use std::slice;

use wasm_bindgen::prelude::*;

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

pub fn canvas_from_raw(canvas_u8: &[u8]) -> &[ColorRgbaU8] {
    let canvas_raw_color = canvas_u8.as_ptr() as *const ColorRgbaU8;
    unsafe {
        slice::from_raw_parts(canvas_raw_color, canvas_u8.len() / 4)
    }
}

pub fn canvas_from_raw_mut(canvas_u8: &mut [u8]) -> &mut [ColorRgbaU8] {
    let canvas_raw_color = canvas_u8.as_mut_ptr() as *mut ColorRgbaU8;
    unsafe {
        slice::from_raw_parts_mut(canvas_raw_color, canvas_u8.len() / 4)
    }
}

#[wasm_bindgen]
pub fn add_buffer(result_buf: &mut [u8], other_buf: &[u8]) {
    if result_buf.len() != other_buf.len() {
        panic!("add_buffer: Buffers aren't the same length!")
    }

    unsafe {
        for i in 0..result_buf.len() {
            *result_buf.get_unchecked_mut(i) += *other_buf.get_unchecked(i)
        }
    }

    // result_buf.iter_mut()
    //     .zip(other_buf.iter())
    //     .for_each(|(result, other)| {
    //         *result += *other;
    //     });
}