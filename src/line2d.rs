use geom::Vec3;
use raster::Rasterize;

pub struct Line2D {
    pub start: Vec3,
    pub end: Vec3,
    pub stroke_width: usize,
}

impl Rasterize for Line2D {
    fn rasterize<F>(&self, raster_width: usize, raster_height: usize, mut render_pixel_at: F)
    where
        F: FnMut(usize, usize),
    {
        if self.stroke_width == 0 {
            return;
        }

        let stroke_delta_pos = self.stroke_width / 2;
        let stroke_delta_neg = self.stroke_width - stroke_delta_pos;

        let from = (self.start.x as i32, self.start.y as i32);

        let to = (self.end.x as i32, self.end.y as i32);

        rasterize_line(from, to, raster_width, raster_height, move |x, y| {
            let min_x = match x {
                x if x <= stroke_delta_neg => 0,
                x => x - stroke_delta_neg,
            };
            // max is exclusive
            let max_x = (x + stroke_delta_pos).min(raster_width);

            //let min_y = if x <= 1 { y } else { y - 1 };
            let min_y = match y {
                y if y <= stroke_delta_neg => 0,
                y => y - stroke_delta_neg,
            };
            let max_y = (y + stroke_delta_pos).min(raster_height);

            for x in min_x..max_x {
                for y in min_y..max_y {
                    render_pixel_at(x, y);
                }
            }
        });
    }
}

/// Rasterizes a line using the Bresenham algorithm.
/// This is used to draw triangles with padding, so they bleed a little into the surrounding
/// space.
/// This is really only useful for painting textures, where a little bleed avoids artifacts
/// due to texture seams.
///
/// Source: https://rosettacode.org/wiki/Bitmap/Bresenham%27s_line_algorithm#C
fn rasterize_line<F>(
    p0: (i32, i32),
    p1: (i32, i32),
    raster_width: usize,
    raster_height: usize,
    mut render_pixel_at: F,
) where
    F: FnMut(usize, usize),
{
    let (x0, y0) = p0;
    let (x1, y1) = p1;

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = if dx > dy { dx } else { -dy } / 2;

    let mut err2;
    let mut x = x0;
    let mut y = y0;

    loop {
        if x >= 0 && x < (raster_width as i32) && y >= 0 && y < (raster_height as i32) {
            render_pixel_at(x as usize, y as usize);
        }

        if x == x1 && y == y1 {
            break;
        }

        err2 = err;

        if err2 > -dx {
            err -= dy;
            x += sx;
        }

        if err2 < dy {
            err += dx;
            y += sy;
        }
    }
}
