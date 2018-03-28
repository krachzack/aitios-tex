
use geom::Triangle;
use ::image::{ImageBuffer, Pixel};
use std::ops::{Deref, DerefMut};

pub trait Rasterize {
    fn rasterize<F>(&self, raster_width: usize, raster_height: usize, render_pixel_at: F)
        where F : FnMut(usize, usize);

    /// Renders a thing already transfomed into image space into the given slice.
    /// The y axis is drawn flipped.
    fn rasterize_to_slice<P, F>(&self, slice: &mut [P], raster_width: usize, raster_height: usize, mut shader_fn: F)
        where F : FnMut(usize, usize) -> P
    {
        self.rasterize(
            raster_width, raster_height,
            |x, y| slice[(raster_height - 1 - y) * raster_width + x] = shader_fn(x, y)
        );
    }

    /// Renders a thing already transfomed into image space into the given image buffer.
    /// The y axis is drawn flipped.
    fn rasterize_to_image<P, C, F>(&self, buf: &mut ImageBuffer<P, C>, shader_fn: F)
        where P: Pixel + 'static,
            C: Deref<Target = [P::Subpixel]> + DerefMut,
            F : Fn(usize, usize) -> P
    {
        let width = buf.width() as usize;
        let height = buf.height() as usize;
        self.rasterize(
            width, height,
            |x, y| buf.put_pixel(x as u32, (height - 1 - y) as u32, shader_fn(x, y))
        );
    }
}

impl<T : Triangle> Rasterize for T {

    /// Fills the triangle with a top-left fill convention, similar to OpenGL.
    /// See: http://forum.devmaster.net/t/advanced-rasterization/6145
    #[allow(non_snake_case)]
    fn rasterize<F>(&self, raster_width: usize, raster_height: usize, mut render_pixel_at: F)
        where F : FnMut(usize, usize)
    {
        /*if self.area() < 0.00000001 {
            return; // ignore zero area triangles
        }*/

        let (v1, v2, v3) = self.positions();

        /*let (v1 , v2, v3) = (
            v1 + Vec3::new(-0.5, -0.5, 0.0),
            v2 + Vec3::new(-0.5, -0.5, 0.0),
            v3 + Vec3::new(-0.5, -0.5, 0.0)
        );*/

        // First draw the minkowski sum of the triangle edges and a 3x3 square by drawing
        // thick lines at the edges.
        // FIXME this is a really weird and inefficient workaround, maybe implement
        // other algorithm with different fill rule / convention
        /*{
            let mut draw3x3 = |x : usize, y : usize| {
                let min_x = match x {
                    0 => 0,
                    1 => 0,
                    x => x - 2
                };
                // max is exclusive
                let max_x = (x + 2).min(raster_width);

                //let min_y = if x <= 1 { y } else { y - 1 };
                let min_y = match y {
                    0 => 0,
                    1 => 0,
                    y => y - 2
                };
                let max_y = (y + 2).min(raster_height);

                for x in min_x..max_x {
                    for y in min_y..max_y {
                        render_pixel_at(x, y);
                    }
                }
            };

            rasterize_line(
                (v1.x.round() as i32, v1.y.round() as i32),
                (v2.x.round() as i32, v2.y.round() as i32),
                raster_width,
                raster_height,
                &mut draw3x3
            );
            rasterize_line(
                (v2.x.round() as i32, v2.y.round() as i32),
                (v3.x.round() as i32, v3.y.round() as i32),
                raster_width,
                raster_height,
                &mut draw3x3
            );
            rasterize_line(
                (v3.x.round() as i32, v3.y.round() as i32),
                (v1.x.round() as i32, v1.y.round() as i32),
                raster_width,
                raster_height,
                &mut draw3x3
            );
        }*/

        // Draw the triangle larger than it actually is, this ensures the triangle
        // will not contain or touch uninitialized pixels, i.e. it adds bleeding
        // to the triangle, which is useful for texture images
        // However, pixels now may be written to multiple times
        /*let padding = 4.0;
        let centroid = self.centroid();

        let mut v1 = v1 - centroid;
        v1.x = v1.x + padding * v1.x.signum() + centroid.x;
        v1.y = v1.y + padding * v1.y.signum() + centroid.y;

        let mut v2 = v2 - centroid;
        v2.x = v2.x + padding * v2.x.signum() + centroid.x;
        v2.y = v2.y + padding * v2.y.signum() + centroid.y;

        let mut v3 = v3 - centroid;
        v3.x = v3.x + padding * v3.x.signum() + centroid.x;
        v3.y = v3.y + padding * v3.y.signum() + centroid.y;
*/

        // 28.4 fixed-point coordinates
        let Y1 = (16.0 * v1.y).round() as i64;
        let Y2 = (16.0 * v2.y).round() as i64;
        let Y3 = (16.0 * v3.y).round() as i64;

        let X1 = (16.0 * v1.x).round() as i64;
        let X2 = (16.0 * v2.x).round() as i64;
        let X3 = (16.0 * v3.x).round() as i64;

        // Deltas
        let DX12 = X1 - X2;
        let DX23 = X2 - X3;
        let DX31 = X3 - X1;

        let DY12 = Y1 - Y2;
        let DY23 = Y2 - Y3;
        let DY31 = Y3 - Y1;

        // Fixed-point deltas
        let FDX12 = DX12 << 4;
        let FDX23 = DX23 << 4;
        let FDX31 = DX31 << 4;

        let FDY12 = DY12 << 4;
        let FDY23 = DY23 << 4;
        let FDY31 = DY31 << 4;

        // Bounding rectangle
        let mut minx = ([X1, X2, X3].iter().min().unwrap() + 0xF) >> 4;
        let mut maxx = ([X1, X2, X3].iter().max().unwrap() + 0xF) >> 4;
        let mut miny = ([Y1, Y2, Y3].iter().min().unwrap() + 0xF) >> 4;
        let mut maxy = ([Y1, Y2, Y3].iter().max().unwrap() + 0xF) >> 4;

        // Clamp to raster size "cull"
        {
            let last_x = raster_width as i64;
            let last_y = raster_height as i64;

            if minx < 0 { minx = 0; }
            if minx > last_x { minx = last_x; }
            if maxx < 0 { maxx = 0; }
            if maxx > last_x { maxx = last_x; }

            if miny < 0 { miny = 0; }
            if miny > last_y { miny = last_y; }
            if maxy < 0 { maxy = 0; }
            if maxy > last_y { maxy = last_y; }
        }

        // Half-edge constants
        let mut C1 = DY12 * X1 - DX12 * Y1;
        let mut C2 = DY23 * X2 - DX23 * Y2;
        let mut C3 = DY31 * X3 - DX31 * Y3;

        // Correct for fill convention
        if DY12 < 0 || (DY12 == 0 && DX12 > 0) { C1 += 1; }
        if DY23 < 0 || (DY23 == 0 && DX23 > 0) { C2 += 1; }
        if DY31 < 0 || (DY31 == 0 && DX31 > 0) { C3 += 1; }

        let mut CY1 = C1 + DX12 * (miny << 4) - DY12 * (minx << 4);
        let mut CY2 = C2 + DX23 * (miny << 4) - DY23 * (minx << 4);
        let mut CY3 = C3 + DX31 * (miny << 4) - DY31 * (minx << 4);

        for y in miny..maxy {
            let mut CX1 = CY1;
            let mut CX2 = CY2;
            let mut CX3 = CY3;

            for x in minx..maxx {
                //if CX1 >= -20 && CX2 >= -20 && CX3 >= -20 {
                if CX1 > 0 && CX2 > 0 && CX3 > 0 {
                    let x = x as usize;
                    let y = y as usize;
                    render_pixel_at(x, y);
                }

                CX1 -= FDY12;
                CX2 -= FDY23;
                CX3 -= FDY31;
            }

            CY1 += FDX12;
            CY2 += FDX23;
            CY3 += FDX31;
        }
    }
}

#[cfg(test)]
mod test {
    extern crate aitios_asset as asset;

    use super::*;
    use geom::{Position, Texcoords, Vec2, Vec3, TupleTriangle, Interpolation, TangentSpace};
    use scene::Mesh;
    use image::{self, Rgb};
    use std::fs::File;
    use std::f32::EPSILON;
    use uv_triangle::triangle_into_uv_image_space;

    /// Takes the mesh triangles and draws the interpolated model positions in UV space
    /// just as in the use case where we want to synthesize a texture.
    #[test]
    fn test_render_positions_in_uv_space() {
        let buddha = &asset::obj::load("tests/assets/buddha.obj")
            .expect("Stanford buddha model could not be loaded for rasterization test")[0];

        let width = 4096_usize;
        let height = 4096_usize;

        let mut world_positions = ImageBuffer::from_pixel(width as u32, height as u32, Rgb { data: [0, 0, 0] });

        buddha.mesh.triangles()
            .filter(|t| t.area() > EPSILON)
            .map(|t| triangle_into_uv_image_space(t, width, height))
            .for_each(|t| t.rasterize_to_image(&mut world_positions, |x, y| {
                let interpolated_position = t.interpolate_at(
                    Vec3::new(x as f32, y as f32, 0.0),
                    |v| v.world_position
                );
                let color = [
                    (interpolated_position.x.fract() * 255.0) as u8,
                    (interpolated_position.y.fract() * 255.0) as u8,
                    (interpolated_position.z.fract() * 255.0) as u8
                ];
                Rgb { data: color }
            }));

        let ref mut fout = File::create("test_position_tex.png").unwrap();

        // Write the contents of this image to the Writer in PNG format.
        image::ImageRgb8(world_positions).save(fout, image::PNG).unwrap();
    }
}
