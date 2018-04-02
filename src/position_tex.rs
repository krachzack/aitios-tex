use geom::{Vec3, Triangle, Position, Interpolation};
use scene::{Entity, Mesh};
use std::f32::EPSILON;
use uv_triangle::triangle_into_uv_image_space;
use line2d::Line2D;
use raster::Rasterize;

pub fn position_tex(entity: &Entity, width: usize, height: usize, island_bleed: usize) -> Vec<Option<Vec3>> {
    let mut positions = vec![None; width * height];

    // 15 for 4096x4096, 9 for 2048x2048, 6 for 1024x1024, 3 for everything below
    //let island_bleed = (width / 1024) * 3 + 3;

    // Minimum of 0.1 squarepixels in UV space
    let uv_space_min_area = 0.1;

    let uv_triangles = entity.mesh
        .triangles()
        .filter(|t| t.area() > EPSILON)
        .map(|t| triangle_into_uv_image_space(t, width, height))
        .filter(|t| t.area() > uv_space_min_area);

    // Before drawing the triangles, draw the outlines in a thick stroke to
    // ensure there will be margins around the UV islands.
    // If there is no padding, blender will display it wrong.
    uv_triangles
        .for_each(|t| {
            let verts = t.iter().map(Position::position);
            let next_verts = t.iter().map(Position::position).cycle().skip(1);

            let lines = verts.zip(next_verts)
                .map(|(start, end)| Line2D { start, end, stroke_width: island_bleed*2 });

            for l in lines {
                l.rasterize_to_slice(&mut positions[..], width, height, |x, y|
                    Some(t.interpolate_at(
                        Vec3::new(x as f32, y as f32, 0.0),
                        |v| v.world_position
                    ))
                )
            }
        });

    // Cannot clone the iterator because the closures cannot be cloned, creating it again
    let uv_triangles = entity.mesh
        .triangles()
        .filter(|t| t.area() > EPSILON)
        .map(|t| triangle_into_uv_image_space(t, width, height))
        .filter(|t| t.area() > uv_space_min_area);

    // Next, draw the insides of the triangles, the real star of the show
    uv_triangles
        .for_each(
            |t| t.rasterize_to_slice(&mut positions[..], width, height, |x, y|
                Some(t.interpolate_at(
                    Vec3::new(x as f32, y as f32, 0.0),
                    |v| v.world_position
                ))
            )
        );

    positions
}
