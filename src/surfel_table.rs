use geom::{Normal, Position};
use geom_tex::{geom_tex, GeomTexel};
use rayon::prelude::*;
use scene::Entity;
use std::f32::EPSILON;
use surf::Surface;

pub fn build_surfel_lookup_table<S>(
    entity: &Entity,
    surf: &Surface<S>,
    surfel_count: usize,
    width: usize,
    height: usize,
    island_bleed: usize,
) -> Vec<Vec<(f32, usize)>>
where
    S: Position + Normal,
    Surface<S>: Sync,
{
    let geom_texels = geom_tex(entity, width, height, island_bleed);

    // Given the normals of a texel and a surfel, cos(theta) must be larger than this
    // to be taken into account.
    // This avoids the back side of a thin surface to influence the front side and vice-versa.
    // for cos(theta) = f32::EPSILON, rotations up to almost theta = 90° are allowed
    const ANGLE_COS_THRESHOLD: f32 = EPSILON;

    geom_texels
        .par_iter()
        .map(|g| {
            g.as_ref()
                .map(
                    |&GeomTexel {
                         position,
                         normal: texel_normal,
                     }| {
                         // FIXME bleeding from other entities may not always be wanted
                        surf.nearest_n_indexes_oriented(
                            position,
                            texel_normal,
                            ANGLE_COS_THRESHOLD,
                            surfel_count,
                        )
                    },
                )
                .unwrap_or_else(Vec::new)
        })
        .collect()
}
