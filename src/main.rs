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

    /// Index of the function in functions range. Value should start from 1 and no more than totalfunction.
    #[arg(long, required(false), default_value("1"))]
    functionindex: u32,

    /// Number of functions in total.
    #[arg(long, required(false), default_value("1"))]
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

    // total number of tiles required in zoomrange
    let mut totaltiles = 0;
    zomr.clone().for_each(|x| {
        totaltiles += 1 << (x * 2);
    });
    let average = totaltiles / args.totalfunction;
    // number of tiles sliced by previous functions
    let tilessliced = average * (args.functionindex - 1);
    // number of tiles to slice in this function, if it's the last
    // function, it should slice all the remaining tiles
    let mut tilestoslice = average;
    if args.functionindex == args.totalfunction {
        tilestoslice = totaltiles - tilessliced;
    }

    // zoom level to start
    let mut startzoomrangetoslice: u8 = *zomr.clone().start();
    // tile index to start
    let mut starttargetrange: u32 = 0;
    // zoom level to stop
    let mut endzoomrangetoslice: u8 = *zomr.clone().end();
    // tile index to stop
    let mut endtargetrange: u32 = 0;

    // total of tiles in previous zoom levels
    let mut tilessum = 0;
    // calculte startzoomrangetoslice and starttargetrange
    for i in zomr.clone() {
        // number of tiles in this zoom level
        let currentzoomtiles = 1 << (i * 2);
        if tilessum + currentzoomtiles > tilessliced {
            startzoomrangetoslice = i;
            starttargetrange = tilessliced - tilessum;
            break;
        } else {
            tilessum += currentzoomtiles;
        }
    }
    tilessum = 0;
    // calculte endzoomrangetoslice and endtargetrange
    for i in zomr.clone() {
        // number of tiles in this zoom level
        let currentzoomtiles = 1 << (i * 2);
        if tilessum + currentzoomtiles >= tilessliced + tilestoslice {
            endzoomrangetoslice = i;
            endtargetrange = tilessliced + tilestoslice - tilessum - 1;
            break;
        } else {
            tilessum += currentzoomtiles;
        }
    }

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
            // if startzoomrangetoslice is the same as endzoomrangetoslice,
            // then tiles to be sliced in this function are from same zoom level
            if startzoomrangetoslice == endzoomrangetoslice {
                if z == endzoomrangetoslice {
                    targetrangetoslice = Some(starttargetrange..=endtargetrange);
                }
            // otherwise, the start zoom level should slice tiles from starttargetrange to end,
            // the end zoom level should slice tiles from 0 to endtargetrange
            } else if z == startzoomrangetoslice {
                if 1 << (z * 2) > 1 {
                    targetrangetoslice = Some(starttargetrange..=(1 << (z * 2)) - 1);
                }
            } else if z == endzoomrangetoslice {
                targetrangetoslice = Some(0..=endtargetrange);
            }
            tile_image
                .iter(&img, targetrangetoslice)
                .for_each(|(sub_img, x, y)| save_subimage(&sub_img, x, y, z, &config).unwrap());
        });
    }
}

#[cfg(test)]
mod main_tests;
