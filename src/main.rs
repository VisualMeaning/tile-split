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

    /// Save the resized files
    #[arg(long, env, action)]
    save_resize: bool,

    /// Index of the function in functions range.
    #[arg(long, required(true))]
    functionindex: u32,

    /// Number of how many functions in total.
    #[arg(long, required(false), default_value("4"))]
    totalfunction: u32,
}

fn main() {
    let args = Args::parse();

    let zomr = if args.zoomrange.is_empty() {
        args.zoomlevel..=args.zoomlevel
    } else {
        args.zoomrange
    };

    let save_resized = args.save_resize;

    // create output folder
    std::fs::create_dir_all(&args.output_dir).unwrap();

    // calculate total number of tiles required in zoomrange
    let mut totaltiles = 0;
    zomr.clone().for_each(|x| {
        totaltiles += 1 << (x * 2);
    });

    // calculte how many tiles to slice
    let average = totaltiles / args.totalfunction;
    let tilessliced= average * (args.functionindex - 1);
    let mut tilestoslice = average;
    if args.functionindex == args.totalfunction {
        tilestoslice = totaltiles - tilessliced;
    }

    // calculate the zoomrange and targetrange for current function
    let mut startzoomrangetoslice: u8 = *zomr.clone().start();
    let mut endzoomrangetoslice: u8= *zomr.clone().end();
    let mut starttargetrange: Option<u32>  = None;
    let mut endtargetrange: Option<u32>  = None;

    (1..args.totalfunction).for_each(|_i| {
        let mut tilessum = 0;
        for j in zomr.clone() {
            let currentzoomtiles = 1 << (j * 2);
            if tilessum + currentzoomtiles > tilessliced {
                startzoomrangetoslice = j;
                starttargetrange = Some(tilessliced - tilessum);
                break;
            } else {
                tilessum += currentzoomtiles;
            }
        }
    });
    (1..args.totalfunction).for_each(|_i| {
        let mut tilessum = 0;
        for j in zomr.clone() {
            let currentzoomtiles = 1 << (j * 2);
            if tilessum + currentzoomtiles >= tilessliced + tilestoslice {
                endzoomrangetoslice = j;
                endtargetrange = Some((tilessliced - tilessum) + tilestoslice - 1);
                break;
            } else {
                tilessum += currentzoomtiles;
            }
        }
    });

    let config = Config {
        tilesize: args.tilesize,
        filename: &args.filename,
        zoomlevel: args.zoomlevel,
        zoomrange: zomr,
        folder: &args.output_dir,
        tileformat: &args.tileformat,
        functionindex: args.functionindex,
        totalfunction: args.totalfunction,
        zoomrangetoslice: (startzoomrangetoslice..=endzoomrangetoslice),
    };

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
            let mut targetrangetoslice: Option<RangeInclusive<u32>> = None;
            if z == endzoomrangetoslice {
                if starttargetrange.is_some() && endtargetrange.is_some() && endtargetrange.unwrap() > starttargetrange.unwrap() {
                    targetrangetoslice = Some(starttargetrange.unwrap()..=endtargetrange.unwrap());
                }
            }
            tile_image
                .iter(&img, targetrangetoslice)
                .for_each(|(sub_img, x, y)| save_subimage(&sub_img, x, y, z, &config).unwrap());
        });
    }
}

#[cfg(test)]
mod main_tests;
