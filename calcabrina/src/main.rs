use std::env;
use std::error::Error;
use std::process;

use image::imageops;
use image::RgbaImage;

use ff4::map;
use ff4::rom;

use calcabrina::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error while parsing arguments: {}", err);
        process::exit(1);
    });

    let rom = rom::Rom::new(&config.filename).unwrap_or_else(|err| {
        println!("Error loading ROM file: {}", err);
        process::exit(1);
    });

    println!("ROM title: {}", rom.title());
    println!("ROM description: {}", rom.description());

    dump_outdoor_tilesets(&rom).unwrap_or_else(|err| {
        println!("Error saving image: {}", err);
        process::exit(1);
    })
}

fn dump_outdoor_tilesets(rom: &rom::Rom) -> Result<(), Box<dyn Error>> {
    dump_outdoor_tileset(&rom, map::OutdoorMap::Overworld, "tileset-overworld.png")?;
    dump_outdoor_tileset(&rom, map::OutdoorMap::Underworld, "tileset-underworld.png")?;
    dump_outdoor_tileset(&rom, map::OutdoorMap::Moon, "tileset-moon.png")?;

    Ok(())
}

fn dump_outdoor_tileset(
    rom: &rom::Rom,
    map: map::OutdoorMap,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Dumping tileset to {}...", filename);

    let tileset = map::OutdoorTileset::new(&rom, map);
    let mut img = RgbaImage::new(128, 128);

    for (i, tile) in tileset.tiles.iter().enumerate() {
        let base_x = (i % 16) * 8;
        let base_y = (i / 16) * 8;

        for (j, palette_index) in tile.pixels.iter().enumerate() {
            let x = (base_x + j % 8) as u32;
            let y = (base_y + j / 8) as u32;

            img.put_pixel(x, y, tileset.palette[*palette_index as usize]);
        }
    }

    let img = imageops::resize(&img, 512, 512, imageops::FilterType::Nearest);
    img.save(filename)?;

    Ok(())
}
