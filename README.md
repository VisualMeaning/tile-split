# Slicing map tiles from illustrations with Rust

The goal is to recreate the functionality of tiles slicing from `imagemagick` in Rust. The slicing time has shown to be significantly reduced, and the built binary can be run as a platform agnostic program.

## Installation for code contribution

Follow the steps from [Rust Documentation](https://doc.rust-lang.org/book/ch01-01-installation.html)

## Building the project

If you are working with the code, run `cargo build` to create a debug build executable file in `target/debug/tile-split`.

If you want to generate a distributable binary, run `cargo build -r` to create a release build executable file in `target/release/tile-split`.

## Running the executable

**Prerequisite for Mac users:** Right click ( control + click ) the `tile-split` program and click Open from the menu. Close the terminal and continue as normal. This step is to create an exception to Mac's security rules for running an unauthorized program. 

### Usage

The binary takes a single PNG file as an input, which must be a square. We tell the binary what zoomlevel the PNG file is via the `--zoomlevel` arg, which is compulsory. To slice a single zoom level PNG, the command is

    tile-split level-5.png --zoomlevel 5

We can also optionally take a PNG for a high zoom level, downsize it to create PNGs of lower zoom levels, and then also slice those into tiles. The `--zoomrange` argument is used to specify which zoomlevels we want to generate tiles for. 

The `--totalfunction` specifies how many sections of work to divide to slice the tiles from zoomrange and the `--functionindex` arguments specifies the section of the tiles we will get. They are mostly used for aws to spread the tiles slicing work equally across mutiple lambda functions. The default value for both `--totalfunction` and `--functionindex` are 1, which means to slice all tiles requested.

    tile-split level-5.png --zoomlevel 5 --zoomrange 2-5 --functionindex 1 --totalfunction 4

This would resize the input single level 5 PNG for zoomlevels 2, 3, 4 5 pngs and slice all tiles from level 2, 3, 4 and first 3 tiles from level 5.

```
tile-split level-5.png --zoomlevel 5 --zoomrange 2-5 --functionindex 2 --totalfunction 4
```

This would resize the input single level 5 PNG for zoomlevels 5 png only and slice tiles 4 - 343 from level 5. 

Note that we cannot upscale an input PNG to higher zoomlevels, only downscale -- so all zoomlevels passed via `--zoomrange` must be equal to or less than the input PNG zoomlevel passed via `--zoomlevel`.

Run `tile-split --help` for more command description.

```
Split input image files into sets of tiles.

Usage: tile-split [OPTIONS] --zoomlevel <ZOOMLEVEL> <FILENAME>

Arguments:
  <FILENAME>  Input PNG filename

Options:
  -l, --zoomlevel <ZOOMLEVEL>     Zoomlevel of input PNG file [env: ZOOMLEVEL=]
  -r, --zoomrange <ZOOMRANGE>...  Zoomrange to slice tiles for
  -o, --output-dir <OUTPUT_DIR>   Location to write output tiles to [env: OUTPUT_DIR=] [default: out]
  -s, --tilesize <TILESIZE>       Dimension of output tiles, in pixels [default: 256]
  -f, --tileformat <TILEFORMAT>   Type of output tiles, currently unused [env: TILEFORMAT=] [default: png]
      --save-resize               Save the resized files [env: SAVE_RESIZE=]
      --functionindex <FUNCTIONINDEX>  Index of the function in functions range. Value should start from 1 and no more than totalfunction [default: 1]
      --totalfunction <TOTALFUNCTION>  Number of functions in total [default: 1]
  -h, --help                      Print help
  -V, --version                   Print version
```


