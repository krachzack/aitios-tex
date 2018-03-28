use geom::Position;
use scene::Entity;
use surf::Surface;
use position_tex::position_tex;
use rayon::prelude::*;

pub fn build_surfel_lookup_table<S>(entity: &Entity, surf: &Surface<S>, surfel_count: usize, width: usize, height: usize, island_bleed: usize) -> Vec<Vec<(f32, usize)>>
    where S : Position,
        Surface<S> : Sync
{
    let positions = position_tex(entity, width, height, island_bleed);

    positions.par_iter()
        .map(
            |p| p.map(
                |p| surf.nearest_n_indexes(p, surfel_count)
            ).unwrap_or_else(Vec::new)
        )
        .collect()
}
