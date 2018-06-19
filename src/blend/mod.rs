mod guided;
mod linear;
mod normal;
mod stops;

pub use self::guided::{
    GuidedBlend,
    BlendType
};
pub use self::linear::blend;
pub use self::normal::{
    blend_normals,
    combine_normals,
    pixel_to_normal,
    normal_to_pixel
};
pub use self::stops::{
    Stop,
    Stops
};
