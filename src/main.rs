use clap::Parser;
use image::{DynamicImage, ImageResult, SubImage};
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::str::FromStr;
use tile_split::{Config, Resizer, TileImage};

fn save_subimage(
    img: &SubImage<&DynamicImage>,
    x: u32,
    y: u32,
    z: u8,
    config: &Config,
) -> ImageResult<()> {
    img.to_image().save(config.folder.join(format!(
        "{z}-{x}-{y}.{fmt}",
        z = z,
        x = x,
        y = y,
        fmt = config.tileformat
    )))
}

fn save_image(img: &DynamicImage, z: u8, config: &Config) -> ImageResult<()> {
    img.save(
        config
            .folder
            .join(format!("{z}.{fmt}", z = z, fmt = config.tileformat)),
    )
}

fn parse_range<T>(arg: &str) -> Result<RangeInclusive<T>, <T as FromStr>::Err>
where
    T: FromStr,
{
    let parts: Vec<&str> = arg.splitn(2, &['-', ' ']).collect::<Vec<&str>>();

    match parts.as_slice() {
        [a] => Ok(RangeInclusive::new(a.parse()?, a.parse()?)),
        [a, b] => Ok(RangeInclusive::new(a.parse()?, b.parse()?)),
        _ => unreachable!(),
    }
}

/// Split input image files into sets of tiles.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input PNG filename.
    filename: PathBuf,

    /// Zoomlevel of input PNG file.
    #[arg(short = 'l', long, env)]
    zoomlevel: u8,

    /// Zoomrange to slice tiles for.
    #[arg(short='r', long, required(false), value_parser = parse_range::<u8>)]
    zoomrange: RangeInclusive<u8>,

    /// Location to write output tiles to.
    #[arg(short, long, env, required(false), default_value("out"))]
    output_dir: PathBuf,

    /// Dimension of output tiles, in pixels.
    #[arg(long, required(false), default_value("256"))]
    tilesize: u32,

    /// Type of output tiles.
    #[arg(long, env, required(false), default_value("png"))]
    tileformat: String,

    /// Subset morton range of tiles to slice.
    #[arg(short='t', long, required(false), value_parser = parse_range::<u32>)]
    targetrange: Option<RangeInclusive<u32>>,

    /// Save the resized files
    #[arg(long, env, action)]
    save_resize: bool,
}

fn main() {
    let args = Args::parse();

    let zomr = if args.zoomrange.is_empty() {
        args.zoomlevel..=args.zoomlevel
    } else {
        args.zoomrange
    };

    let config = Config {
        tilesize: args.tilesize,
        filename: &args.filename,
        zoomlevel: args.zoomlevel,
        zoomrange: zomr,
        folder: &args.output_dir,
        tileformat: &args.tileformat,
        targetrange: args.targetrange,
    };
    let save_resized = args.save_resize;

    // create output folder
    std::fs::create_dir_all(config.folder).unwrap();

    // instantiate TileImage
    let tile_image = TileImage { config: &config };
    let image = &tile_image.open_img().unwrap();

    // resize (and save)
    let resized_images = config.resize_range(image);

    if save_resized {
        resized_images.for_each(|(img, z)| save_image(&img, z, &config).unwrap())
    } else {
        // save each sliced image
        resized_images.for_each(|(img, z)| {
            tile_image
                .iter(&img)
                .for_each(|(sub_img, x, y)| save_subimage(&sub_img, x, y, z, &config).unwrap());
        });
    }
}

#[cfg(test)]
mod main_tests;
