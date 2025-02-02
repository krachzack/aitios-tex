//!
//! Provides functionality for processing surfel data into substance density textures.
//!

use self::SubstanceFilter::*;
use geom::Vertex;
use image::{ImageBuffer, Pixel, Rgba};
use scene::Entity;
use sim::SurfelData;
use surf;
use surfel_table::build_surfel_lookup_table;
use std::f32::NEG_INFINITY;

type Surface = surf::Surface<surf::Surfel<Vertex, SurfelData>>;

pub enum SubstanceFilter {
    /// When combining n surfels into a texel, take the unweighted average of substance.
    Flat,
    /// When combining n surfels into a texel do a weighted average, give the nearest
    /// texel the highest influence, gradually decreasing until the last surfel with influence 0
    Smooth,
}

pub struct Density {
    substance_idx: usize,
    tex_width: usize,
    tex_height: usize,
    island_bleed: usize,
    min_density: f32,
    max_density: f32,
    /// Color to use for locations in the texture unused by the mesh
    undefined_color: Rgba<u8>,
    min_color: Rgba<u8>,
    max_color: Rgba<u8>,
    filtering: SubstanceFilter,
}

impl Density {
    pub fn new(
        substance_idx: usize,
        tex_width: usize,
        tex_height: usize,
        island_bleed: usize,
        min_density: f32,
        max_density: f32,
        undefined_color: Rgba<u8>,
        min_color: Rgba<u8>,
        max_color: Rgba<u8>,
        filtering: SubstanceFilter,
    ) -> Self {
        Density {
            substance_idx,
            tex_width,
            tex_height,
            island_bleed,
            min_density,
            max_density,
            undefined_color,
            min_color,
            max_color,
            filtering,
        }
    }

    pub fn build_table(&self, entity: &Entity, surf: &Surface) -> Vec<Vec<(f32, usize)>> {
        build_surfel_lookup_table(
            entity,
            surf,
            4,
            self.tex_width,
            self.tex_height,
            self.island_bleed,
        )
    }

    pub fn collect(&self, entity: &Entity, surf: &Surface) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        //let position_tex = position_tex(entity, self.tex_width, self.tex_height, self.island_bleed);

        self.collect_with_table(
            surf,
            &build_surfel_lookup_table(
                entity,
                surf,
                4,
                self.tex_width,
                self.tex_height,
                self.island_bleed,
            ),
        )
    }

    pub fn collect_with_table(
        &self,
        surf: &Surface,
        table: &Vec<Vec<(f32, usize)>>,
    ) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_fn(self.tex_width as u32, self.tex_height as u32, |x, y| {
            let x = x as usize;
            let y = y as usize;
            let surfels = &table[y * self.tex_width + x];
            let density = match surfels.len() {
                0 => None,
                // Single surfel, no filtering
                1 => Some(surf.samples[surfels[0].1].data().substances[self.substance_idx]),
                _ => Some(match self.filtering {
                    Flat => self.density_at_idxs(surf, surfels),
                    Smooth => self.dist_sqr_ratio_weighted(surf, surfels),
                })
            };

            match density {
                None => self.undefined_color,
                Some(density) => {
                    let alpha = density.max(self.min_density).min(self.max_density)
                        / (self.max_density - self.min_density);

                    self.min_color.map2(&self.max_color, |c0, c1| {
                        ((1.0 - alpha) * (c0 as f32) + alpha * (c1 as f32)) as u8
                    })
                }
            }
        })
    }

    fn weighted_avg(&self, surf: &Surface, close_surfels: &Vec<(f32, usize)>, weights: impl Clone + Iterator<Item = f32>) -> f32 {
        let one_over_weights_sum = weights.clone().sum::<f32>().recip();
        let scaled_weights = weights.map(|w| one_over_weights_sum * w);
        close_surfels.iter()
            .map(|&(_, surfel_idx)| surf.samples[surfel_idx].data().substances[self.substance_idx])
            .zip(scaled_weights)
            .map(|(substance, weight)| substance * weight)
            .sum::<f32>()
    }
    
    
    fn dist_sqr_ratio_weighted(&self, surf: &Surface, close_surfels: &Vec<(f32, usize)>) -> f32 {
        // The maximally distant surfel still has some influence, 
        const MIN_WEIGHT : f32 = 1.0;
        // If a surfel completely coincides with the texel position, it has MIN_WEIGHT+RANGE influence
        const RANGE : f32 = 5.0;
        let dists = close_surfels.iter().map(|&(dist, _)| dist);
        let max_dist = dists.clone().fold(NEG_INFINITY, f32::max);
        let max_dist_inv = max_dist.recip();
        let weights = dists.map(|d| (max_dist - d) * max_dist_inv * RANGE + MIN_WEIGHT);
        self.weighted_avg(surf, close_surfels, weights)
    }

    /*fn inv_distance_weighted(&self, surf: &Surface, close_surfels: &Vec<(f32, usize)>) -> f32 {
        // Super expensive, running sqrt twice
        let weights = close_surfels.iter().map(|&(dist, _)| dist.sqrt().recip());
        self.weighted_avg(surf, close_surfels, weights)
    }*/

    fn density_at_idxs(&self, surf: &Surface, close_surfels: &Vec<(f32, usize)>) -> f32 {
        let one_over_n = (close_surfels.len() as f32).recip();

        one_over_n
            * close_surfels
                .iter()
                .map(|&(_dist, idx)| surf.samples[idx].data().substances[self.substance_idx])
                .sum::<f32>()
    }
}
