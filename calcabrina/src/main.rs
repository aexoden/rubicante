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

    dump_composed_outdoor_tileset(&rom, map::OutdoorMap::Overworld, "tileset-composed-overworld.png")?;
    dump_composed_outdoor_tileset(&rom, map::OutdoorMap::Underworld, "tileset-composed-underworld.png")?;
    dump_composed_outdoor_tileset(&rom, map::OutdoorMap::Moon, "tileset-composed-moon.png")?;

    Ok(())
}

fn draw_outdoor_tile(img: &mut RgbaImage, tileset: &map::OutdoorTileset, index: usize, base_x: usize, base_y: usize) {
    for (i, palette_index) in tileset.tiles[index].pixels.iter().enumerate() {
        let x = (base_x + i % 8) as u32;
        let y = (base_y + i / 8) as u32;

        img.put_pixel(x, y, tileset.palette[*palette_index as usize]);
    }
}

fn dump_outdoor_tileset(
    rom: &rom::Rom,
    map: map::OutdoorMap,
    filename: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Dumping tileset to {}...", filename);

    let tileset = map::OutdoorTileset::new(&rom, map);
    let mut img = RgbaImage::new(128, 128);

    for i in 0..tileset.tiles.len() {
        let base_x = (i % 16) * 8;
        let base_y = (i / 16) * 8;

        draw_outdoor_tile(&mut img, &tileset, i, base_x, base_y);
    }

    let img = imageops::resize(&img, 512, 512, imageops::FilterType::Nearest);
    img.save(filename)?;

    Ok(())
}

fn dump_composed_outdoor_tileset(rom: &rom::Rom, map: map::OutdoorMap, filename: &str) -> Result<(), Box<dyn Error>> {
    println!("Dumping composed tileset to {}...", filename);

    let tileset = map::OutdoorTileset::new(&rom, map);
    let mut img = RgbaImage::new(256, 128);

    for (i, composition) in tileset.composition.iter().enumerate() {
        let base_x = (i % 16) * 16;
        let base_y = (i / 16) * 16;

        draw_outdoor_tile(&mut img, &tileset, composition.upper_left, base_x, base_y);
        draw_outdoor_tile(&mut img, &tileset, composition.upper_right, base_x + 8, base_y);
        draw_outdoor_tile(&mut img, &tileset, composition.lower_left, base_x, base_y + 8);
        draw_outdoor_tile(&mut img, &tileset, composition.lower_right, base_x + 8, base_y + 8);
    }

    let img = imageops::resize(&img, 1024, 512, imageops::FilterType::Nearest);
    img.save(filename)?;

    Ok(())
}
