use clap::Parser;
use image::{DynamicImage, ImageResult, SubImage};
use std::path::PathBuf;
use std::str::FromStr;
use std::{ops::RangeInclusive, path::Path};
use tile_split::{Config, Resizer, TileImage};

fn save_subimage(
    img: &SubImage<&DynamicImage>,
    x: u32,
    y: u32,
    z: u8,
    folder: &Path,
    tileformat: &str,
) -> ImageResult<()> {
    img.to_image().save(folder.join(format!(
        "{z}-{x}-{y}.{fmt}",
        z = z,
        x = x,
        y = y,
        fmt = tileformat
    )))
}

fn save_image(img: &DynamicImage, z: u8, folder: &Path, tileformat: &str) -> ImageResult<()> {
    img.save(folder.join(format!("{z}.{fmt}", z = z, fmt = tileformat)))
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

    /// Save the resized files
    #[arg(long, env, action)]
    save_resize: bool,

    /// Index of the function in functions range. Value should start from 1 and no more than totalfunction.
    #[arg(long, required(false), default_value("1"))]
    functionindex: u32,

    /// Number of functions in total.
    #[arg(long, required(false), default_value("1"))]
    totalfunction: u32,
}

fn main() {
    let args = Args::parse();

    let save_resized = args.save_resize;

    // create output folder
    std::fs::create_dir_all(&args.output_dir).unwrap();

    let config = Config::new(
        &args.filename,
        args.tilesize,
        args.zoomlevel,
        args.zoomrange,
        args.functionindex,
        args.totalfunction,
    );

    // instantiate TileImage
    let tile_image = TileImage { config: &config };
    let image = &tile_image.open_img().unwrap();

    // resize (and save)
    let resized_images = config.resize_range(image);

    if save_resized {
        resized_images
            .for_each(|(img, z)| save_image(&img, z, &args.output_dir, &args.tileformat).unwrap())
    } else {
        // save each sliced image
        resized_images.for_each(|(img, z)| {
            let mut targetrangetoslice: Option<RangeInclusive<u32>> = None;
            // if startzoomrangetoslice is the same as endzoomrangetoslice,
            // then tiles to be sliced in this function are from same zoom level
            if config.startzoomrangetoslice == config.endzoomrangetoslice {
                if z == config.endzoomrangetoslice {
                    targetrangetoslice = Some(config.starttargetrange..=config.endtargetrange);
                }
            // otherwise, the start zoom level should slice tiles from starttargetrange to end,
            // the end zoom level should slice tiles from 0 to endtargetrange
            } else if z == config.startzoomrangetoslice {
                if 1 << (z * 2) > 1 {
                    targetrangetoslice = Some(config.starttargetrange..=(1 << (z * 2)) - 1);
                }
            } else if z == config.endzoomrangetoslice {
                targetrangetoslice = Some(0..=config.endtargetrange);
            }
            tile_image
                .iter(&img, targetrangetoslice)
                .for_each(|(sub_img, x, y)| {
                    save_subimage(&sub_img, x, y, z, &args.output_dir, &args.tileformat).unwrap()
                });
        });
    }
}

#[cfg(test)]
mod main_tests;
