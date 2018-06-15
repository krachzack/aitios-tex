//!
//! Provides functionality for processing surfel data into substance density textures.
//!

use geom::Vertex;
use scene::Entity;
use image::{ImageBuffer, Pixel, Rgba};
use surf;
use surfel_table::build_surfel_lookup_table;
use sim::SurfelData;

type Surface = surf::Surface<surf::Surfel<Vertex, SurfelData>>;

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
    max_color: Rgba<u8>
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
        max_color: Rgba<u8>
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
            max_color
        }
    }

    pub fn build_table(&self, entity: &Entity, surf: &Surface) -> Vec<Vec<(f32, usize)>> {
        build_surfel_lookup_table(entity, surf, 4, self.tex_width, self.tex_height, self.island_bleed)
    }

    pub fn collect(&self, entity: &Entity, surf: &Surface) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        //let position_tex = position_tex(entity, self.tex_width, self.tex_height, self.island_bleed);

        self.collect_with_table(surf, &build_surfel_lookup_table(entity, surf, 4, self.tex_width, self.tex_height, self.island_bleed))

        // REVIEW if this cannot guarantee correct same output as input order,
        //        this does not work
        /*let densities : Vec<_> = position_tex.par_iter()
            .map(|p| {
                p.map(|p| {
                    self.density_at(surf, p)
                })
            })
            .collect();

        ImageBuffer::from_fn(
            self.tex_width as u32,
            self.tex_height as u32,
            |x, y| {
                let x = x as usize;
                let y = y as usize;
                match densities[y * self.tex_width + x] {
                    None => self.undefined_color,
                    Some(density) => {
                        let alpha = density.max(self.min_density).min(self.max_density) /
                                    (self.max_density - self.min_density);

                        self.min_color.map2(
                            &self.max_color,
                            |c0, c1| ((1.0 - alpha) * (c0 as f32) + alpha * (c1 as f32)) as u8
                        )
                    }
                }
            }
        )*/
    }

    pub fn collect_with_table(&self, surf: &Surface, table: &Vec<Vec<(f32, usize)>>) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        ImageBuffer::from_fn(
            self.tex_width as u32,
            self.tex_height as u32,
            |x, y| {
                let x = x as usize;
                let y = y as usize;
                let surfels = &table[y * self.tex_width + x];
                let density = if surfels.is_empty() { None } else { Some(self.density_at_idxs(surf, surfels)) };

                match density {
                    None => self.undefined_color,
                    Some(density) => {
                        let alpha = density.max(self.min_density).min(self.max_density) /
                                    (self.max_density - self.min_density);

                        self.min_color.map2(
                            &self.max_color,
                            |c0, c1| ((1.0 - alpha) * (c0 as f32) + alpha * (c1 as f32)) as u8
                        )
                    }
                }
            }
        )
    }

    fn density_at_idxs(&self, surf: &Surface, close_surfels: &Vec<(f32, usize)>) -> f32 {
        // REVIEW should the lookup be limited to surfels of the same entity?
        /*let sample_radius = close_surfels.iter()
                .map(|&(dist, _)| dist)
                .fold(NEG_INFINITY, f32::max);*/

        // This is inspired by photon mapping, see: https://graphics.stanford.edu/courses/cs348b-00/course8.pdf
        // > 1, characterizes the filter
        //let k = 2.7;

        let one_over_n = (close_surfels.len() as f32).recip();

        one_over_n * close_surfels.iter()
            .map(|&(_dist, idx)| surf.samples[idx].data().substances[self.substance_idx])
            .sum::<f32>()

        /*one_over_n * surfels.iter()
            .map(|&(dist, surfel)| (1.0 - (dist / (k * sample_radius))) * surfel.data().substances[self.substance_idx])
            .sum::<f32>()*/
    }
}
