use clap::Parser;
use image::{DynamicImage, ImageResult, SubImage};
use std::fs::File;
use std::io::Write;
use rayon::prelude::*;
use std::path::PathBuf;
use std::str::FromStr;
use std::{ops::RangeInclusive, path::Path};
use tile_split::{Config, Error, Resizer, TileImage};

fn save_subimage(
    sub: &SubImage<&DynamicImage>,
    x: &u32,
    y: &u32,
    z: u8,
    folder: &Path,
    config: &Config,
) -> Result<(), Error> {
    let path = folder.join(format!("{z}-{x}-{y}.png", z = z, x = x, y = y));
    let png = oxipng::RawImage::new(
        config.tilesize,
        config.tilesize,
        oxipng::ColorType::RGBA,
        oxipng::BitDepth::Eight,
        sub.to_image().into_raw(),
    )?
    .create_optimized_png(&oxipng::Options::from_preset(2))?;
    let mut file = File::create(path)?;
    file.write_all(&png)?;

    Ok(())
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

    /// Subset morton range of tiles to slice.
    #[arg(short='t', long, required(false), value_parser = parse_range::<u32>)]
    targetrange: Option<RangeInclusive<u32>>,

    /// Save the resized files
    #[arg(long, env, action)]
    save_resize: bool,
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
        args.targetrange,
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
        resized_images
            .collect::<Vec<(DynamicImage, u8)>>()
            .par_iter()
            .for_each(|(img, z)| {
                let mut targetrangetoslice: Option<RangeInclusive<u32>> = None;
                // if startzoomrangetoslice is the same as endzoomrangetoslice,
                // then tiles to be sliced in this function are from same zoom level
                if config.startzoomrangetoslice == config.endzoomrangetoslice {
                    if *z == config.endzoomrangetoslice {
                        targetrangetoslice = Some(config.starttargetrange..=config.endtargetrange);
                    }
                // otherwise, the start zoom level should slice tiles from starttargetrange to end,
                // the end zoom level should slice tiles from 0 to endtargetrange
                } else if *z == config.startzoomrangetoslice {
                    if 1 << (z * 2) > 1 {
                        targetrangetoslice = Some(config.starttargetrange..=(1 << (z * 2)) - 1);
                    }
                } else if *z == config.endzoomrangetoslice {
                    targetrangetoslice = Some(0..=config.endtargetrange);
                }
                tile_image
                    .iter(&img, targetrangetoslice).collect::<Vec<(SubImage<&DynamicImage>, u32, u32)>>()
                    .par_iter()
                    .for_each(|(sub_img, x, y)| {
                        save_subimage(&sub_img, x, y, *z, &args.output_dir, &config).unwrap()
                    });
            });
    }
}

#[cfg(test)]
mod main_tests;
