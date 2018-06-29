use geom::{Triangle, TupleTriangle, FromVertices, Texcoords, Position, Normal, Vec3, Vec2};
use geom::prelude::ElementWise;

/// Consumes the given triangle and returns a triangle that reports the UV coordinates
/// of its vertices as the X/Y coordinates of its position with Z always being zero.
///
/// This is useful to paint textures.
///
/// Note that the vertex order might be changed so that the triangle is CCW on the X/Y
/// plane with Z pointing in positive direction.
pub fn triangle_into_uv_image_space<T, V>(tri: T, image_width: usize, image_height: usize) -> TupleTriangle<UvVtx>
        where T : Triangle<Vertex = V>,
            V : Position + Texcoords + Normal
{
    let scale = Vec2::new(image_width as f32, image_height as f32);

    let ((texcoord0, texcoord1, texcoord2), (worldpos0, worldpos1, worldpos2), (worldnormal0, worldnormal1, worldnormal2)) = {
        let (v0, v1, v2) = tri.vertices();

        let texcoords = (
            v0.texcoords().mul_element_wise(scale),
            v1.texcoords().mul_element_wise(scale),
            v2.texcoords().mul_element_wise(scale)
        );

        let positions = (
            v0.position(),
            v1.position(),
            v2.position()
        );

        let normals = (
            v0.normal(),
            v1.normal(),
            v2.normal()
        );

        if is_ccw(&texcoords) {
            (texcoords, positions, normals)
        } else {
            // Flip order if would be pointing downwards in uv space
            (flip(texcoords), flip(positions), flip(normals))
        }
    };

    TupleTriangle::new(
        UvVtx { uv_position: texcoord0, world_position: worldpos0, world_normal: worldnormal0 },
        UvVtx { uv_position: texcoord1, world_position: worldpos1, world_normal: worldnormal1 },
        UvVtx { uv_position: texcoord2, world_position: worldpos2, world_normal: worldnormal2 },
    )
}

/// Checks if the given tuple of 2D position represents
/// a triangle with counter-clockwise winding order in
/// a Y-up coordinate system.
///
/// Uses the shoelace method for calculating the area of
/// arbitrary polygons that will return positive or negative
/// area depending on winding order.
fn is_ccw(texcoords: &(Vec2, Vec2, Vec2)) -> bool {
    (texcoords.1.x - texcoords.0.x) * (texcoords.1.y + texcoords.0.y) +
    (texcoords.2.x - texcoords.1.x) * (texcoords.2.y + texcoords.1.y) +
    (texcoords.0.x - texcoords.2.x) * (texcoords.0.y + texcoords.2.y) <= 0.0
}

/// Swaps two elements in a triple to swap the order.
fn flip<T>(tuple: (T, T, T)) -> (T, T, T) {
    (tuple.0, tuple.2, tuple.1)
}

// Vertex that exposes uv coordinates as position, filling in 0.0 as Z coordinate
#[derive(Debug, Copy, Clone)]
pub struct UvVtx {
    pub uv_position: Vec2,
    pub world_normal: Vec3,
    pub world_position: Vec3
}

impl Position for UvVtx {
    // Triangles in UV space
    fn position(&self) -> Vec3 {
        self.uv_position.extend(0.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn ccw_coordinates() -> (Vec2, Vec2, Vec2) {
        (Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0))
    }

    #[test]
    fn ccw() {
        assert!(
            is_ccw(&ccw_coordinates()),
            "Triangle should report as counter-clockwise."
        );
    }

    #[test]
    fn cw() {
        println!("{:?}", flip(ccw_coordinates()));

        assert!(
            !is_ccw(&flip(ccw_coordinates())),
            "Triangle should report as clockwise after flipping the counter-clockwise one."
        );
    }
}
