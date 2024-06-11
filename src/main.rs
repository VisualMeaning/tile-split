use clap::Parser;
use image::io::Reader as ImageReader;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::str::FromStr;
use tile_split::Config;

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
    zoomrange: Option<RangeInclusive<u8>>,

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

    /// PNG compression preset.
    #[arg(long, env, default_value_if("tileformat", "png", "2"), value_parser(clap::value_parser!(u8).range(0..7)))]
    preset: Option<u8>,

    /// Save the resized files
    #[arg(long, env, action)]
    save_resize: bool,
}

fn main() {
    let args = Args::parse();

    if args.preset.is_some() && &args.tileformat != "png" {
        eprintln!(
            "Error: The --preset argument cannot be used with --tileformat set to '{}'",
            &args.tileformat
        );
        std::process::exit(2);
    }

    // create output folder
    std::fs::create_dir_all(&args.output_dir).unwrap();

    let config = Config::new(
        &args.filename,
        args.tilesize,
        args.zoomlevel,
        args.zoomrange,
        args.targetrange,
        args.preset,
    );

    // load image
    let mut reader = match ImageReader::open(config.filename) {
        Ok(reader) => reader,
        Err(e) => panic!("Problem opening the image: {:?}", e),
    };
    // Default memory limit of 512MB is too small for level 6+ PNGs
    reader.no_limits();
    let loaded_image = match reader.decode() {
        Ok(reader_image) => {
            if reader_image.width() != reader_image.height() {
                panic!("Image is not square!")
            } else {
                reader_image
            }
        }
        Err(e) => panic!("Problem decoding the image: {:?}", e),
    };

    // resize
    let resized_images = config.resize_range(&loaded_image);

    if args.save_resize {
        resized_images.into_iter().for_each(|(img, z)| {
            img.save_image(z, &args.output_dir, &args.tileformat)
                .unwrap()
        })
    } else {
        // save each sliced image
        resized_images.into_par_iter().for_each(|(img, z)| {
            let tiles = img.slice_tiles(z);
            tiles.into_par_iter().for_each(|tile| {
                let img = tile.to_subimage();
                if &args.tileformat == "png" {
                    let oxipng = tile.convert_to_oxipng(img);
                    let path = &args.output_dir.join(format!(
                        "{z}-{x}-{y}.png",
                        z = z,
                        x = tile.x,
                        y = tile.y
                    ));
                    let mut file = File::create(path).unwrap();
                    file.write_all(&oxipng).unwrap();
                } else {
                    let path = &args.output_dir.join(format!(
                        "{z}-{x}-{y}.{fmt}",
                        z = z,
                        x = tile.x,
                        y = tile.y,
                        fmt = &args.tileformat
                    ));
                    img.to_image().save(path).unwrap();
                }
            });
        });
    }
}

#[cfg(test)]
mod main_tests;
