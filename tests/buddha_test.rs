extern crate aitios_geom as geom;
extern crate aitios_tex as tex;
extern crate aitios_sim as sim;
extern crate aitios_scene as scene;
extern crate aitios_asset as asset;
extern crate aitios_surf as surf;
extern crate image;

use geom::Vec3;
use scene::Entity;
use sim::{TonSourceBuilder, SurfelData, Simulation};
use surf::{SurfaceBuilder, SurfelSampling};
use asset::obj;
use scene::{Mesh, MaterialBuilder};
use tex::Density;
use image::Rgba;
use std::fs::File;
use std::iter;
use std::rc::Rc;

/// Minimal example of a simulation.
/// Running on a single entity and collecting densities after each of 5 iterations,
/// then persisting the resulting density masks.
#[ignore]
#[test]
fn buddha_test() {
    let buddha = &obj::load("tests/assets/buddha.obj")
        .expect("Could not load test geometry")[0];

    let prototype_surfel_data = SurfelData {
        entity_idx: 0,
        delta_straight: 1.0,
        delta_parabolic: 0.4,
        delta_flow: 1.0,
        substances: vec![0.0],
        deposition_rates: vec![1.0],
        rules: vec![]
    };

    let surface = SurfaceBuilder::new()
        .sampling(SurfelSampling::MinimumDistance(0.1))
        .sample_triangles(buddha.mesh.triangles(), &prototype_surfel_data)
        .build();

    let source = TonSourceBuilder::new()
        .p_straight(0.0)
        .p_parabolic(1.0)
        .p_flow(0.0)
        .substances(&vec![1.0])
        .hemisphere_shaped(Vec3::new(0.0, 0.0, 0.0), 15.0)
        .emission_count(10000)
        .interaction_radius(0.2)
        .parabola_height(0.1)
        .pickup_rates(vec![0.5])
        .build();

    let mut sim = Simulation::new(
        vec![source],
        buddha.mesh.triangles(),
        surface,
        vec![]
    );

    let density = Density::new(
        0,    // substance index
        512, // tex width
        512, // tex height
        2,   // island bleed in pixels
        0.0,  // surfel density equivalent to min color
        1.0,  // surfel density equivalent to max color
        Rgba { data: [ 0, 0, 255, 255 ] },     // color for texture space unused by the entity
        Rgba { data: [ 255, 255, 255, 255 ] }, // color for 0.0 density
        Rgba { data: [ 0, 0, 0, 255 ] }        // color for 1.0 density
    );

    for iteration in 0..10 {
        sim.run();
        let updated_surface = sim.surface();

        let density_tex = density.collect(buddha, updated_surface);
        let filename = format!("tests/iteration-{}-density.png", iteration);

        let ref mut fout = File::create(&filename).unwrap();

        // Write the contents of this image to the Writer in PNG format.
        image::ImageRgba8(density_tex).write_to(fout, image::PNG).unwrap();

        let buddha_with_density_tex = Entity {
            name: "DensityBuddha".to_string(),
            material: Rc::new(MaterialBuilder::new()
                .name(format!("Iteration{}DensityAsDiffuse", iteration))
                .diffuse_color_map(filename)
                .build()
            ),
            mesh: buddha.mesh.clone()
        };

        obj::save(
            iter::once(&buddha_with_density_tex),
            Some(format!("tests/iteration-{}-density.obj", iteration)),
            Some(format!("tests/iteration-{}-density.mtl", iteration))
        ).expect("Failed creating OBJ/MTL with density texture set at diffuse map");
    }
}
