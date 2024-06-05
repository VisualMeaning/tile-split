use clap::Parser;
use rayon::prelude::*;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;
use std::{ops::RangeInclusive, path::Path};
use tile_split::{Config, Error, Tile, TileImage};

fn save_tile(tile: &Tile, z: u8, folder: &Path, format: &str) -> Result<(), Error> {
    let path = folder.join(format!(
        "{z}-{x}-{y}.{fmt}",
        z = z,
        x = tile.x,
        y = tile.y,
        fmt = format
    ));
    tile.img.to_image().save(path)?;

    Ok(())
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

    // instantiate and load image
    let image = TileImage::new(&config, None);

    // resize (and save)
    let resized_images =
        RangeInclusive::new(config.startzoomrangetoslice, config.endzoomrangetoslice)
            .into_par_iter()
            .map(|x: u8| {
                let t_size = config.tilesize << x;
                (image.resize(t_size, t_size), x)
            });

    if args.save_resize {
        resized_images.for_each(|(img, z)| {
            img.save_image(z, &args.output_dir, &args.tileformat)
                .unwrap()
        })
    } else {
        // save each sliced image
        resized_images.for_each(|(img, z)| {
            img.iter_tiles(z)
                .collect::<Vec<Tile>>()
                .par_iter()
                .for_each(|tile| {
                    if &args.tileformat == "png" {
                        let oxipng = tile.convert_to_oxipng();
                        let path = &args.output_dir.join(format!(
                            "{z}-{x}-{y}.png",
                            z = z,
                            x = tile.x,
                            y = tile.y
                        ));
                        let mut file = File::create(path).unwrap();
                        file.write_all(&oxipng).unwrap();
                    } else {
                        save_tile(tile, z, &args.output_dir, &args.tileformat).unwrap()
                    }
                });
        });
    }
}

#[cfg(test)]
mod main_tests;
