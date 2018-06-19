use image::{Rgba, Primitive};

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
