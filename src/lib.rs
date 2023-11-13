mod config;
mod image;
mod resize;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

pub use self::config::Config;
pub use self::image::TileImage;
pub use self::resize::Resizer;
