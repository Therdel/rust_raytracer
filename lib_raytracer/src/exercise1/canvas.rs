use crate::raytracing::color::*;

pub struct Canvas {
    pixels: Vec<ColorRgbU8>,
    width: usize,
    height: usize
}

pub struct CanvasStripe<'a> {
    pixel_row: &'a mut [ColorRgbU8],
    y_coord: usize,
}

impl CanvasStripe<'_> {
    pub fn set_pixel(&mut self, x: usize, color: &ColorRgb) {
        self.pixel_row[x] = color.quantize();
    }

    pub fn get_y_coord(&self) -> usize {
        self.y_coord
    }
}

impl Canvas {
    pub fn new(canvas_dimensions: (usize, usize), background: ColorRgb) -> Canvas {
        Canvas {
            pixels: vec![background.quantize(); canvas_dimensions.0 * canvas_dimensions.1],
            width: canvas_dimensions.0,
            height: canvas_dimensions.1
        }
    }

    pub fn borrow_stripes_mut(&mut self) -> impl Iterator<Item=CanvasStripe> {
        let max_y_index = self.height - 1;

        let row_stripes = self.pixels
            .chunks_mut(self.width)
            .enumerate()
            .map(move |(y_coord, pixel_row)| {
                let y_inverted = max_y_index - y_coord;
                CanvasStripe { pixel_row, y_coord: y_inverted }
            });
        row_stripes
    }

    #[allow(dead_code)]
    pub fn set_pixel(&mut self, x: usize, y: usize, color: &ColorRgb) {
        let max_y_index = self.height - 1;
        let y_inverted = max_y_index - y;
        let pos = x + self.width * y_inverted;
        self.pixels[pos] = color.quantize();
    }

    pub fn get_pixels(&self) -> &[ColorRgbU8] {
        &self.pixels
    }

    pub fn get_width(&self) -> usize {
        self.width
    }

    pub fn get_height(&self) -> usize {
        self.height
    }
}