use geom::{Triangle, TupleTriangle, FromVertices, TangentSpace, Texcoords, Position, Normal, Vec3, Vec2};
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

        let tex_tri_normal = TupleTriangle::new(
            texcoords.0.extend(0.0),
            texcoords.1.extend(0.0),
            texcoords.2.extend(0.0)
        ).normal();

        if tex_tri_normal.z <= 0.0 {
            (texcoords, positions, normals)
        } else {
            // Flip order if would be pointing downwards in uv space
            (
                (
                    texcoords.0,
                    texcoords.2,
                    texcoords.1
                ),
                (
                    positions.0,
                    positions.2,
                    positions.1
                ),
                (
                    normals.0,
                    normals.2,
                    normals.1
                )
            )
        }
    };

    TupleTriangle::new(
        UvVtx { uv_position: texcoord0, world_position: worldpos0, world_normal: worldnormal0 },
        UvVtx { uv_position: texcoord1, world_position: worldpos1, world_normal: worldnormal1 },
        UvVtx { uv_position: texcoord2, world_position: worldpos2, world_normal: worldnormal2 },
    )
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
