mod guided;
mod linear;
mod normal;
mod stops;

pub use self::guided::{BlendType, GuidedBlend};
pub use self::linear::blend;
pub use self::normal::{blend_normals, combine_normals, normal_to_pixel, pixel_to_normal};
pub use self::stops::{Stop, Stops};
