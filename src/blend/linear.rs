use image::{Rgba, Primitive};

/// Linearly blends the four components of `color0` towards
/// `color1` using the blending factor `alpha`.
///
/// The alpha channel of the colors is treated the same as the
/// color channel, linearly blending and not influencing the
/// blending factor.
pub fn blend<P>(color0: Rgba<P>, color1: Rgba<P>, alpha: f32) -> Rgba<u8>
    where P : Primitive + Into<f32> + 'static
{
    let Rgba { data: color0 } = color0;
    let Rgba { data: color1 } = color1;

    Rgba {
        data: [
            ((1.0 - alpha) * color0[0].into() + alpha * color1[0].into()) as u8,
            ((1.0 - alpha) * color0[1].into() + alpha * color1[1].into()) as u8,
            ((1.0 - alpha) * color0[2].into() + alpha * color1[2].into()) as u8,
            ((1.0 - alpha) * color0[3].into() + alpha * color1[3].into()) as u8
        ]
    }
}

/*use std::ops::{Mul, Sub, Add};

/// Linearly interpolates between edge0 and edge1 according
/// to the given alpha value between 0 and 1.
fn lerp<E, A>(edge0: E, edge1: E, alpha: A) -> <<<f64 as Sub<A>>::Output as Mul<E>>::Output as Add>::Output
    where E : Mul<A>,
          f64 : Sub<A>,
          <f64 as Sub<A>>::Output : Mul<E>,
          A : Mul<E>,
          <<f64 as Sub<A>>::Output as Mul<E>>::Output : Add<<<f64 as Sub<A>>::Output as Mul<E>>::Output>
{
    let one_minus_alpha = 1.0 - alpha;

    /*let edge0 : f64 = edge0.into();
    let edge1 : f64 = edge1.into();
    let alpha : f64 = alpha.into();*/

    one_minus_alpha * edge0 + alpha * edge1
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lerp_f32() {
        let edge0 = -100.0;
        let edge1 = 100.0;

        assert_eq!(
            edge0,
            lerp(edge0, edge1, 0.0),
            "Alpha of 0.0 should yield value equal to edge0"
        );

        assert_eq!(
            edge1,
            lerp(edge0, edge1, 1.0),
            "Alpha of 1.0 should yield value equal to edge1"
        );

        assert_eq!(
            (edge0 + edge1) / 2.0,
            lerp(edge0, edge1, 0.5),
            "Alpha of 0.5 should yield value exactly in between edge0 and edge1"
        );
    }

    fn lerp_u32() {
        let edge0 = 0_f32;
        let edge1 = 255_f32;

        assert_eq!(
            edge0,
            lerp(edge0, edge1, 0.0),
            "Alpha of 0.0 should yield value equal to edge0"
        );
    }
}*/
