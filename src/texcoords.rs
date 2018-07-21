//!
//! Implements operations for sampling textures.
//!

use image::GenericImage;
use std::f32::NAN;

/// Samples the closest pixel to the given UV cooridnates.
///
/// The UV coordinates follow the OpenGL convention with
/// v axis pointing upward in the image plane. That is,
/// the UV coordinates (0, 1) represent the pixel in the top
/// left.
///
/// If the coordinate exceeds the range [0,1], a mirroring
/// repeat scheme is applied, e.g. 1.2 is equal to 0.8,
/// but 2.2 is equal to 0.2.
pub fn sample<I: GenericImage>(image: &I, u: f32, v: f32) -> I::Pixel {
    let u = repeat_mirror(u);
    let v = repeat_mirror(v);

    let (x, y) = uv_to_offset(u, v, image.width(), image.height());

    image.get_pixel(x, y)
}

/// Makes the given UV coordinate oscillate between 0 and 1, making
/// a texture repeat and flip direction after each repetition.
/// E.g. 1.2 is equal to 0.8, but 2.2 and -0.2 are equal to 0.2.
/// Infinities or NAN return NAN.
pub fn repeat_mirror(coord: f32) -> f32 {
    if coord.is_infinite() {
        return NAN;
    }

    let coord = coord.abs();

    let whole = coord.trunc() as i64;
    let fractional = coord.fract();

    if whole & 1 == 0 {
        fractional
    } else {
        1.0 - fractional
    }
}

/// Converts the center point of the given pixel offset within the given
/// image dimensions to uv coordinates in range [0,1].
/// The returned coordinates follow OpenGL conventions with (1, 1) represeting
/// the top right and (0, 0) representing bottom left. In other words,
/// U/V is with Y axis pointing upward, while pixel offsets are assumed
/// in scanline order, that is, with Y axis pointing downward.
pub fn offset_to_uv(x: u32, y: u32, width: u32, height: u32) -> (f32, f32) {
    assert!(x < width && y < height);

    let x = x as f32;
    let y = y as f32;
    let width = width as f32;
    let height = height as f32;

    ((x + 0.5) / width, (height - y - 0.5) / height)
}

pub fn uv_to_offset(u: f32, v: f32, width: u32, height: u32) -> (u32, u32) {
    assert!(u >= 0.0 && u < 1.0 && v >= 0.0 && v < 1.0);

    let width = width as f32;
    let height = height as f32;

    (
        (u * width).floor() as u32,
        ((1.0 - v) * height).floor() as u32,
    )
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn offset_calculation() {
        assert_eq!(
            offset_to_uv(0, 0, 123, 123),
            (0.5 / 123.0, (123.0 - 0.5) / 123.0)
        );
    }

    #[test]
    fn mirroring() {
        assert_ulps_eq!(repeat_mirror(-0.2), 0.2);
        assert_ulps_eq!(repeat_mirror(0.2), 0.2);
        assert_ulps_eq!(repeat_mirror(1.2), 0.8);
        assert_ulps_eq!(repeat_mirror(2.2), 0.2);

        assert!(repeat_mirror(0.0 / 0.0).is_nan());
        assert!(repeat_mirror(1.0 / 0.0).is_nan());
    }
}
