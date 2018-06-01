use image::{GenericImage, Rgba, ImageBuffer, Pixel, Luma, Primitive};
use texcoords::{offset_to_uv, sample};

pub struct GuidedBlend<I> {
    stops: Vec<Stop<I>>,
}

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
        let blend = Self {
            stops: stops.into_iter().collect()
        };

        if blend.stops.is_empty() {
            panic!("Tried to create GuidedBlend with an empty iterator of stops, which is undefined");
        }

        blend
    }

    pub fn perform<G>(&self, guide: &G) -> ImageBuffer<Rgba<u8>, Vec<u8>>
        where G : GenericImage,
            <<G as GenericImage>::Pixel as Pixel>::Subpixel : Into<f32> + 'static
    {
        let (output_width, output_height) = self.output_dimensions();

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

    /// Gets the dimensions of the maximum sample size as the output dimension.
    /// Panics if no stop is defined.
    fn output_dimensions(&self) -> (u32, u32) {
        self.stops.iter()
            .map(|s| s.sample.dimensions())
            .max()
            .unwrap()
    }
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

fn blend<P : Primitive + Into<f32> + 'static>(color0: Rgba<P>, color1: Rgba<P>, alpha: f32) -> Rgba<u8> {
    let Rgba { data: color0 } = color0;
    let Rgba { data: color1 } = color1;

    Rgba {
        data: [
            ((1.0 - alpha) * (color0[0].into()) + alpha * (color1[0].into())) as u8,
            ((1.0 - alpha) * (color0[1].into()) + alpha * (color1[1].into())) as u8,
            ((1.0 - alpha) * (color0[2].into()) + alpha * (color1[2].into())) as u8,
            ((1.0 - alpha) * (color0[3].into()) + alpha * (color1[3].into())) as u8
        ]
    }
}

#[cfg(test)]
mod test {

}
