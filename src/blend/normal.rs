use image::{Rgba, Primitive};
use geom::{Vec3, Vec2, InnerSpace, ElementWise};

/// Interpolates between two normals. At maximum alpha, no detail of
/// normal0 will be retained.
///
/// This behavior differs from `combine_normals`, where normals are not
/// blended but added together by distinguishing between base and detail
/// normal.
pub fn blend_normals<P>(normal0: Rgba<P>, normal1: Rgba<P>, alpha: f32) -> Rgba<u8>
    where P : Primitive + Into<f32> + 'static
{
    let normal0 = pixel_to_normal(normal0);
    let normal1 = pixel_to_normal(normal1);

    // partial derivatives
    let pd0 = Vec2::new(normal0.x, normal0.y) / normal0.z;
    let pd1 = Vec2::new(normal1.x, normal1.y) / normal1.z;

    // blend derivatives and blend back to normal
    let pd = (1.0 - alpha) * pd0 + alpha * pd1;
    // REVIEW the [blog post](http://blog.selfshadow.com/publications/blending-in-detail/).
    //        uses 1.0 and adds pd0 and pd1 together instead of blending
    //        using 0.5 leads to similar results as when adding
    let normal = pd.extend(0.5).normalize();

    normal_to_pixel(normal)
}

/// Combines two normals by reorienting the first normal towards the second
/// using the approach described in this
/// [blog post](http://blog.selfshadow.com/publications/blending-in-detail/).
/// This captures the behavior where the second normal overlaays the base normal.
///
/// The approach leaves the base normal still visible with the detail normal
/// only adding details
fn combine_normals<P>(base: Rgba<P>, detail: Rgba<P>) -> Rgba<u8>
    where P : Primitive + Into<f32> + 'static
{
    let base = (pixel_to_normal(base) + Vec3::new(1.0, 1.0, 1.0)) / 2.0;
    let detail = (pixel_to_normal(detail) + Vec3::new(1.0, 1.0, 1.0)) / 2.0;

    let t = base * 2.0 + Vec3::new(-1.0, -1.0, 0.0);
    let u = detail.mul_element_wise(Vec3::new(-2.0, -2.0, 2.0)) + Vec3::new(1.0, 1.0, -1.0);
    let r  = t * t.dot(u) - u * t.z;

    normal_to_pixel(r.normalize())
}

/// Converts a tangent space normal map texel to a normalized tangent-space vector.
fn pixel_to_normal<P>(texel: Rgba<P>) -> Vec3
    where P : Primitive + Into<f32> + 'static
{
    let Rgba { data } = texel;

    // This scales everything in range 0..1
    let inv_scale = P::max_value().into().recip();
    let normal = inv_scale * Vec3::new(data[0].into(), data[1].into(), data[2].into());

    // Map 0..1 to -1..1
    normal * 2.0 - Vec3::new(1.0, 1.0, 1.0)
}

fn normal_to_pixel(normal: Vec3) -> Rgba<u8> {
    // Map -1..1 to 0..1 and scale by bit depth
    let Vec3 { x, y, z } = (normal * 0.5 + Vec3::new(0.5, 0.5, 0.5)) * (u8::max_value() as f32);
    Rgba {
        data: [ x as u8, y as u8, z as u8, 255 ]
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use image::{open, GenericImage, ImageBuffer};

    #[test]
    fn test_partial_derivative_cone_and_bumps() {
        let cone = open("tests/assets/normal_cone.png")
            .expect("Could not load cone-like normal map for normal blending test");

        let bumpy = open("tests/assets/normal_bumpy.png")
            .expect("Could not load cone-like normal map for normal blending test");

        assert_eq!(cone.dimensions(), bumpy.dimensions());

        let black = Rgba { data: [0, 0, 0, 255] };

        let mut inbetween = ImageBuffer::from_pixel(cone.width(), cone.height(), black);
        inbetween.pixels_mut()
            .zip(cone.pixels().zip(bumpy.pixels()))
            .for_each(|(t, ((_, _, n0), (_, _, n1)))| *t = blend_normals(n0, n1, 0.5));
        inbetween.save("tests/generated/normal_cone_bumpy_blent_pd.png")
            .expect("Result of normal blending could not be written");
    }

    #[test]
    fn test_rnm_normal_combination() {
        let cone = open("tests/assets/normal_cone.png")
            .expect("Could not load cone-like normal map for normal blending test");

        let bumpy = open("tests/assets/normal_bumpy.png")
            .expect("Could not load cone-like normal map for normal blending test");

        let black = Rgba { data: [0, 0, 0, 255] };

        let mut inbetween = ImageBuffer::from_pixel(cone.width(), cone.height(), black);
        inbetween.pixels_mut()
            .zip(cone.pixels().zip(bumpy.pixels()))
            .for_each(|(t, ((_, _, n0), (_, _, n1)))| *t = combine_normals(n0, n1));

        inbetween.save("tests/generated/normal_cone_bumpy_combined_rnm.png")
            .expect("Result of normal combination could not be written");
    }
}
