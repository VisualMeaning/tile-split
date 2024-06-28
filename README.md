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

We can also optionally take a PNG for a high zoom level, downsize it to create PNGs of lower zoom levels, and then also slice those into tiles. The `--zoomrange` argument is used to specify which zoomlevels we want to generate tiles for. The `--targetrange` argument specifies subset of tiles to slice, if not provided the full range of tiles is sliced.

    tile-split level-5.png --zoomlevel 5 --zoomrange 2-5 --targetrange 0-44

This would generate tiles for zoomlevels 2, 3, 4 and 5 from the single level 5 PNG and slice the first subset of 45 tiles for the whole zoom range.

For example, for a chunk size of 300 tiles per round, the arguments would be

    tile-split level-5.png --zoomlevel 5 --zoomrange 0-5 --targetrange 0-300
    tile-split level-5.png --zoomlevel 5 --zoomrange 0-5 --targetrange 300-600
    tile-split level-5.png --zoomlevel 5 --zoomrange 0-5 --targetrange 600-900
    tile-split level-5.png --zoomlevel 5 --zoomrange 0-5 --targetrange 900-1200
    tile-split level-5.png --zoomlevel 5 --zoomrange 0-5 --targetrange 1200-1365

Note that we cannot upscale an input PNG to higher zoomlevels, only downscale -- so all zoomlevels passed via `--zoomrange` must be equal to or less than the input PNG zoomlevel passed via `--zoomlevel`.

If `--parentzoomlevel` is provided, it indicates that the input PNG has the dimension of the `--zoomlevel` value and is sliced from an whole png of dimension `--parentzoomlevel`. `--indexforzoom` indicates the index of the input PNG among all the sliced ones (Note the index needs to follow Z-curve). If `--parentzoomlevel` is provided, `--zoomrange` and `--targetrange` will be ignored and the code will split the input PNG to all tiles and give it the correct names that corresponds to the parent zoom level location. These arguments are designed to be run by multiple lambda function to slice level 6 and level 7 PNGs.

For example, if we have four level 5 dimension PNGs sliced from one whole level 6 PNG, by running

    tile-split 5-0-0.png --zoomlevel 5 --parentzoomlevel 6 --indexforzoom 0
    tile-split 5-1-0.png --zoomlevel 5 --parentzoomlevel 6 --indexforzoom 1
    tile-split 5-0-1.png --zoomlevel 5 --parentzoomlevel 6 --indexforzoom 2
    tile-split 5-1-1.png --zoomlevel 5 --parentzoomlevel 6 --indexforzoom 3
This will output all 4096 tiles from level 6 with correct names.

Run `tile-split --help` for more command description.

```
Split input image files into sets of tiles.

Usage: tile-split [OPTIONS] --zoomlevel <ZOOMLEVEL> <FILENAME>

Arguments:
  <FILENAME>  Input PNG filename

Options:
  -l, --zoomlevel <ZOOMLEVEL>
          Zoomlevel of input PNG file [env: ZOOMLEVEL=]
  -p, --parentzoomlevel <PARENTZOOMLEVEL>
          Parent zoomlevel of input sub PNG file
  -i, --indexforzoom <INDEXFORZOOM>
          Index of the input sub PNG [default: 0]
  -r, --zoomrange <ZOOMRANGE>
          Zoomrange to slice tiles for
  -o, --output-dir <OUTPUT_DIR>
          Location to write output tiles to [env: OUTPUT_DIR=] [default: out]
      --tilesize <TILESIZE>
          Dimension of output tiles, in pixels [default: 256]
      --tileformat <TILEFORMAT>
          Type of output tiles [env: TILEFORMAT=] [default: png]
  -t, --targetrange <TARGETRANGE>
          Subset morton range of tiles to slice
      --preset <PRESET>
          PNG compression preset [env: PRESET=]
      --save-resize
          Save the resized files [env: SAVE_RESIZE=]
  -h, --help
          Print help
  -V, --version
          Print version
```


