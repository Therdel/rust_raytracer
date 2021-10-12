use lib_raytracer::raytracing::*;
use lib_raytracer::raytracing::color::ColorRgb;

#[no_mangle]
pub extern "C" fn add_one(x: i32) -> i32 {
    x + 1
}

#[no_mangle]
pub extern "C" fn vec_size(x: i32) -> i32 {
    let mut vec = Vec::new();
    for _ in 0..x {
        vec.push(2);
    }
    vec.iter().sum()
}

#[no_mangle]
pub extern "C" fn color_sum() -> f32 {
    let color: ColorRgb = color::Color::urple();
    color.sum()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
