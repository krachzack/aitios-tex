use geom::{Position, Normal, InnerSpace};
use scene::Entity;
use surf::Surface;
use geom_tex::{GeomTexel, geom_tex};
use rayon::prelude::*;
use std::f32::EPSILON;

pub fn build_surfel_lookup_table<S>(entity: &Entity, surf: &Surface<S>, surfel_count: usize, width: usize, height: usize, island_bleed: usize) -> Vec<Vec<(f32, usize)>>
    where S : Position + Normal,
        Surface<S> : Sync
{
    let geom_texels = geom_tex(entity, width, height, island_bleed);

    // Given the normals of a texel and a surfel, cos(theta) must be larger than this
    // to be taken into account.
    // This avoids the back side of a thin surface to influence the front side and vice-versa.
    const ANGLE_COS_THRESHOLD : f32 = EPSILON;

    geom_texels.par_iter()
        .map(
            |g| g.as_ref().map(
                |&GeomTexel { position, normal: texel_normal }| {
                    let mut nearest = surf.nearest_n_indexes(position, surfel_count);
                    nearest.retain(|&(_, idx)| {
                        let surfel_normal = surf.samples[idx].normal();
                        surfel_normal.dot(texel_normal) > ANGLE_COS_THRESHOLD
                    });
                    nearest
                }
            ).unwrap_or_else(Vec::new)
        )
        .collect()
}
