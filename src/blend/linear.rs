use image::{GenericImage, Rgba, ImageBuffer, Pixel, Luma, Primitive};
use texcoords::{offset_to_uv, sample};

#[derive(Debug)]
pub struct GuidedBlend<I> {
    stops: Vec<Stop<I>>,
}

#[derive(Debug)]
pub struct Stop<I> {
    sample: I,
    cenith: f32
}

impl<I> Stop<I> {
    pub fn new(cenith: f32, sample: I) -> Self {
        Self { sample, cenith }
    }
}

impl<I> GuidedBlend<I>
    where I : GenericImage,
        <<I as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static
{
    pub fn new(stops: impl IntoIterator<Item = Stop<I>>) -> Self {
        let mut stops : Vec<Stop<I>> = stops.into_iter().collect();

        if stops.is_empty() {
            panic!("Tried to create GuidedBlend with an empty iterator of stops, which is undefined");
        }

        if stops.iter()
            .any(|s| s.cenith.is_infinite())
        {
            let ceniths = stops.iter().map(|s| s.cenith).collect::<Vec<_>>();
            panic!("Some ceniths were NaN/Infinity during guided blend construction: {:?}", ceniths);
        }

        // Sort for fast lookup of cenith before and after
        stops.sort_by(|a, b| {
            a.cenith.partial_cmp(&b.cenith)
                .unwrap() // NaN or infinite would have panicked before, comparison is safe
        });

        Self { stops }
    }

    /// Blends using the given guide texture to create a new image buffer
    /// with the same size as the guide.
    ///
    /// The guide can have different dimensions than the stop samples.
    ///
    /// Only the luminosity of the given guide texture is used. If it has
    /// an alpha channel, it is ignored.
    pub fn perform<G>(&self, guide: &G) -> ImageBuffer<Rgba<u8>, Vec<u8>>
        where G : GenericImage,
            <<G as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static
    {
        let (output_width, output_height) = guide.dimensions();

        ImageBuffer::from_fn(
            output_width,
            output_height,
            |x, y| {
                let (u, v) = offset_to_uv(x, y, output_width, output_height);
                let guide = pixel_to_alpha(sample(guide, u, v).to_luma());

                let (stop_before, stop_after) = self.stops_before_after(guide);

                let sample0 = sample(&stop_before.sample, u, v).to_rgba();
                let sample1 = sample(&stop_after.sample, u, v).to_rgba();

                let edge0 = stop_before.cenith;
                let edge1 = stop_after.cenith;

                let alpha = if edge0 == edge1 {
                    0.0
                } else {
                    (guide - edge0) / (edge1 - edge0)
                };

                blend(sample0, sample1, alpha)
            }
        )
    }

    fn stops_before_after(&self, guide: f32) -> (&Stop<I>, &Stop<I>) {
        let mut stop_iter = self.stops.iter();
        let mut last_stop = stop_iter.next().unwrap(); // always at least one

        while let Some(stop) = stop_iter.next() {
            if last_stop.cenith <= guide && stop.cenith > guide {
                return (last_stop, stop);
            }
            last_stop = stop;
        }

        // If only one stop specified, it is the last stop and returned twice.
        // If no stop has a cenith greater than the guide,
        // repeat the stop with the highest cenith
        (last_stop, last_stop)
    }
}

fn pixel_to_alpha<T>(pixel: Luma<T>) -> f32
    where T : Primitive + Into<f32> + 'static
{
    // FIXME channels4 marked for deprecation
    let (luminosity, _, _, _) = pixel.channels4();
    let luminosity = luminosity.into();
    // FIXME assuming T to be u8
    let max_luminosity : f32 = 255.0;//P::Subpixel::max_value().into();

    luminosity / max_luminosity
}

fn blend<P>(color0: Rgba<P>, color1: Rgba<P>, alpha: f32) -> Rgba<u8>
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn guided_blend() {
        let black : ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_pixel(2, 2, Rgba { data: [ 0, 0, 0, 255 ] });
        let white : ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_pixel(2, 2, Rgba { data: [ 255, 255, 255, 255 ] });
        let stops = vec![
            Stop::new(0.0, black),
            Stop::new(1.0, white),
        ];
        let blend = GuidedBlend::new(stops);

        let guide : Vec<u8> = vec![
            0_u8, 0_u8, 0_u8, 0_u8,
            255_u8, 255_u8, 255_u8, 255_u8,
            128_u8, 128_u8, 128_u8, 128_u8,
            128_u8, 128_u8, 128_u8, 128_u8
        ];
        let guide : ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            2, 2,
            guide
        ).unwrap();

        let blent = blend.perform(&guide);

        // Since edge0 is black and edge1 is white, result should be identical to guide,
        // except for alpha channel.
        for x in 0..2 {
            for y in 0..2 {
                let blent_pixel = blent.get_pixel(x, y);
                let guide_pixel = guide.get_pixel(x, y);
                // Colors should be like guide map since stop 0 is black and stop 1 is white
                assert_eq!(blent_pixel.to_rgb(), guide_pixel.to_rgb());
                // Except alpha which is 255 in both stops
                assert_eq!(255, blent_pixel.channels()[3]);
            }
        }
    }
}
