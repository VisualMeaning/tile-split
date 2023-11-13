use clap::Parser;
use tile_split::{Config, TileImage};
use image::{DynamicImage, SubImage};


fn save_subimage(img: &SubImage<&DynamicImage>, x: u32, y: u32, config: &Config) {
    img.to_image().save(format!(
        "{p}/{z}_{x}_{y}.{fmt}",
        p=config.folder,
        z=config.zoomlevel,
        x=x,
        y=y,
        fmt=config.tileformat)
    ).unwrap();
}

/// Split input image files into sets of tiles.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input PNG filename.
    filename: String, 

    /// Zoomlevel of input PNG file
    #[arg(short='l', long, env)]
    zoomlevel: u8,

    /// Zoomrange to slice tiles for, currently unused.
    #[arg(short='r', long, required(false), num_args=1.., value_delimiter = ' ')]
    zoomrange: Vec<u8>,

    /// Location to write output tiles to.
    #[arg(short, long, env, required(false), default_value("out"))]
    output_dir: String,

    /// Dimension of output tiles, in pixels.
    #[arg(short='s', long, required(false), default_value("256"))]
    tilesize: u32,

    /// Type of output tiles, currently unused.
    #[arg(short='f', long, env, required(false), default_value("png"))]
    tileformat: String,
}

fn main() {
    let args = Args::parse();

    let config = Config {
            tilesize: args.tilesize,
            filename: &args.filename,
            zoomlevel: args.zoomlevel,
            folder: &args.output_dir,
            tileformat: &args.tileformat,
    };
    
    // create output folder
    std::fs::create_dir_all(config.folder).unwrap();

    // instantiate TileImage
    let tile_image = TileImage{
        config: &config,
    };
    let image = &tile_image.open_img().unwrap();

    // save each sliced image
    tile_image
        .iter(image)
        .for_each(|(sub_img, x, y)| save_subimage(&sub_img, x, y, &config));
}
