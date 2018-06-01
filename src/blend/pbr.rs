//!
//! Implements the synthesis of PBR maps by blending multiple together.
//!

use image::{GenericImage, ImageBuffer, Pixel, Rgb, Luma, Primitive};
use std::f32::NAN;
use geom::{Vec3, prelude::*};

/// Blends the first given normal map towards the second.
/// The blending factors are specified by the luminosity of the
/// corresponding pixel in the guide map.
///
/// While all maps may have different dimensions, all three must
/// have the same aspect ratio.
pub fn blend_by<N, G>(map0: &N, map1: &N, guide: &G) -> ImageBuffer<Rgb<u8>, Vec<u8>>
    where N : GenericImage, G : GenericImage,
        <<N as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static,
        <<G as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static
{
    let (map0_width, map0_height) = map0.dimensions();
    let (map1_width, map1_height) = map1.dimensions();

    // Choose output size by largest input normal map dimensions
    let blent_width = map0_width.max(map1_width);
    let blent_height = map0_height.max(map1_height);

    ImageBuffer::from_fn(
        blent_width,
        blent_height,
        |x, y| {
            let (u, v) = offset_to_uv(x, y, blent_width, blent_height);

            let map0 = rgb_to_normal(sample(map0, u, v).to_rgb());
            let map1 = rgb_to_normal(sample(map1, u, v).to_rgb());
            let alpha = pixel_to_alpha(sample(guide, u, v).to_luma());

            let normal = blend_normals(map0, map1, alpha);

            normal_to_rgb(normal)
        }
    )
}

/// Blends the first given normal map towards the second.
/// The blending factors are specified by the luminosity of the
/// corresponding pixel in the guide map.
///
/// While all maps may have different dimensions, all three must
/// have the same aspect ratio.
fn blend_normal_by<N, G>(normal0: &N, normal1: &N, guide: &G) -> ImageBuffer<Rgb<u8>, Vec<u8>>
    where N : GenericImage,
        G : GenericImage,
        <<N as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static,
        <<G as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static
{
    unimplemented!()
}

fn pixel_to_alpha<T>(pixel: Luma<T>) -> f32
    where T : Primitive + Into<f32> + 'static
{
    let (luminosity, _, _, _) = pixel.channels4();
    let luminosity = luminosity.into();
    // FIXME assuming T to be u8
    let scale : f32 = 255.0;//P::Subpixel::max_value().into();

    luminosity / scale
}

fn rgb_to_normal<T>(rgb: Rgb<T>) -> Vec3
    where T: Primitive + Into<f32> + 'static
{
    let (r, g, b, _) = rgb.channels4();
    let r : f32 = r.into();
    let g : f32 = g.into();
    let b : f32 = b.into();

    Vec3::new(r, g, b)
        .normalize() // scale and normalize
}

fn normal_to_rgb(normal: Vec3) -> Rgb<u8> {
    let Vec3 { x, y, z } = normal;
    let r = (x * 255.0).round() as u8;
    let g = (y * 255.0).round() as u8;
    let b = (z * 255.0).round() as u8;
    Rgb { data: [ r, g, b ] }
}

fn blend_normals(normal0: Vec3, normal1: Vec3, alpha: f32) -> Vec3 {
    // this does not really work well, find a better method
    let interpolated = (1.0 - alpha) * normal0 + alpha * normal1;

    interpolated.normalize()
}

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
fn sample<I : GenericImage>(image: &I, u: f32, v: f32) -> I::Pixel {
    let u = repeat_mirror(u);
    let v = repeat_mirror(v);

    let (x, y) = uv_to_offset(u, v, image.width(), image.height());

    image.get_pixel(x, y)
}

/// Makes the given UV coordinate repeat mirroring.
/// E.g. 1.2 is equal to 0.8, but 2.2 and -0.2 are equal to 0.2.
/// Infinities or NAN return NAN.
fn repeat_mirror(mut coord: f32) -> f32 {
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
fn offset_to_uv(x: u32, y: u32, width: u32, height: u32) -> (f32, f32) {
    assert!(x < width && y < height);

    let x = x as f32;
    let y = y as f32;
    let width = width as f32;
    let height = height as f32;

    (
        (x + 0.5) / width,
        (height - y - 0.5) / height
    )
}

fn uv_to_offset(u: f32, v: f32, width: u32, height: u32) -> (u32, u32) {
    assert!(u >= 0.0 && u < 1.0 && v >= 0.0 && v < 1.0);

    let width = width as f32;
    let height = height as f32;

    (
        (u * width).floor() as u32,
        ((1.0 - v) * height).floor() as u32
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

        assert!(repeat_mirror(0.0/0.0).is_nan());
        assert!(repeat_mirror(1.0/0.0).is_nan());
    }
}
