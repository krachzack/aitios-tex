use geom::{Vec3, Triangle, Position, Interpolation};
use scene::{Entity, Mesh};
use uv_triangle::triangle_into_uv_image_space;
use line2d::Line2D;
use raster::Rasterize;

#[derive(Clone)]
pub struct GeomTexel {
    pub position: Vec3,
    pub normal: Vec3,
    // The ratio between the area in world space to the area in texture space.
    //pub scale: f32
}

pub fn geom_tex(entity: &Entity, width: usize, height: usize, island_bleed: usize) -> Vec<Option<GeomTexel>> {
    let mut geom_texels = vec![None; width * height];

    // 15 for 4096x4096, 9 for 2048x2048, 6 for 1024x1024, 3 for everything below
    //let island_bleed = (width / 1024) * 3 + 3;

    //let min_area = 0.15; // At least 15% of a pixel

    let uv_triangles = entity.mesh
        .triangles()
        .map(|t| triangle_into_uv_image_space(t, width, height));

    // Before drawing the triangles, draw the outlines in a thick stroke to
    // ensure there will be margins around the UV islands.
    // If there is no padding, blender will display it wrong.
    if island_bleed > 0 {
        uv_triangles
            .for_each(|t| {
                let verts = t.iter().map(Position::position);
                let next_verts = t.iter().map(Position::position).cycle().skip(1);

                let lines = verts.zip(next_verts)
                    .map(|(start, end)| Line2D { start, end, stroke_width: island_bleed*2 });

                for l in lines {
                    l.rasterize_to_slice(&mut geom_texels[..], width, height, |x, y|
                        Some(GeomTexel {
                            position: t.interpolate_at(
                                Vec3::new(x as f32, y as f32, 0.0),
                                |v| v.world_position
                            ),
                            normal: t.interpolate_at(
                                Vec3::new(x as f32, y as f32, 0.0),
                                |v| v.world_normal
                            ),
                            //scale: unimplemented!("Texel-to-world scale compensation currently unimplemented")
                        })
                    )
                }
            });
    }

    // Cannot clone the iterator because the closures cannot be cloned, creating it again
    let uv_triangles = entity.mesh
        .triangles()
        // For triangle interiors, only select the triangles with a relevant area,
        // that is, triangles where the vertices do not lie on the same line (or almost)
        .filter(|t| !t.is_colinear())
        .map(|t| triangle_into_uv_image_space(t, width, height));

    // Next, draw the insides of the triangles, the real star of the show
    uv_triangles
        .for_each(
            |t| t.rasterize_to_slice(&mut geom_texels[..], width, height, |x, y|
                Some(GeomTexel {
                    position: t.interpolate_at(
                        Vec3::new(x as f32, y as f32, 0.0),
                        |v| v.world_position
                    ),
                    normal: t.interpolate_at(
                        Vec3::new(x as f32, y as f32, 0.0),
                        |v| v.world_normal
                    ),
                    //scale: unimplemented!("Texel-to-world scale compensation currently unimplemented")
                })
            )
        );

    geom_texels
}
