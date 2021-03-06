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

// #[wasm_bindgen]
// pub fn merge_interlaced_buffers(result_buf: &mut [u8], other_buffers: &[&[u8]],
//                                 buffer_width: usize, buffer_height: usize) {
//     let amount_buffers = other_buffers.len();
//     const AMOUNT_PIXEL_BYTES: usize = 4;
//     let row_len_bytes = buffer_width * AMOUNT_PIXEL_BYTES;
//
//     enum RenderVariant {
//         None,
//         UnsafeLinewise,
//         SafeLinewise
//     }
//     use RenderVariant::*;
//     const VARIANT: RenderVariant = UnsafeLinewise;
//
//     match VARIANT {
//         None => {},
//         UnsafeLinewise => {
//             let result_buf_raw = result_buf.as_mut_ptr();
//             for y in 0..buffer_height {
//                 let buffer_index = y % amount_buffers;
//                 let row_offset = y * row_len_bytes;
//
//                 unsafe {
//                     let buffer = *other_buffers.get_unchecked(buffer_index);
//                     let buffer_row_raw = (buffer.as_ptr() + row_offset) as *const u8;
//                     let result_row_raw = result_buf_raw + row_offset;
//
//                     std::ptr::copy(buffer_row_raw, result_row_raw, row_len_bytes);
//                 }
//             }
//         },
//         SafeLinewise => {
//             let result_rows = result_buf.chunks_exact_mut(row_len_bytes)
//                 .enumerate();
//             for (y, row) in result_rows {
//                 let buffer_index = y % amount_buffers;
//                 let buffer = other_buffers[buffer_index];
//                 let row_offset = y * row_len_bytes;
//                 let buffer_row: &[u8] = buffer.get(row_offset..(row_offset+row_len_bytes)).unwrap();
//                 row.copy_from_slice(buffer_row);
//             }
//         }
//     }
// }
