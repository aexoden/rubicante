use image::Rgba;

use super::rom;
use super::rom_map::record;

const PIXELS_PER_TILE: usize = 64;
const COMPOSED_TILES_PER_TILESET: usize = 128;
const TILES_PER_TILESET: usize = 256;

pub enum OutdoorMap {
    Overworld,
    Underworld,
    Moon,
}

pub struct TileComposition {
    pub upper_left: usize,
    pub upper_right: usize,
    pub lower_left: usize,
    pub lower_right: usize,
}

pub struct OutdoorTile {
    pub pixels: Vec<u8>,
}

pub struct OutdoorTileset {
    pub composition: Vec<TileComposition>,
    pub palette: Vec<Rgba<u8>>,
    pub tiles: Vec<OutdoorTile>,
}

impl OutdoorTileset {
    pub fn new(rom: &rom::Rom, map: OutdoorMap) -> OutdoorTileset {
        let map_index = match map {
            OutdoorMap::Overworld => 0,
            OutdoorMap::Underworld => 1,
            OutdoorMap::Moon => 2,
        };

        let upper_values = rom.read_bytes(record::OUTDOOR_TILESET_UPPER_VALUES, map_index);
        let lower_values = rom.read_bytes(record::OUTDOOR_TILESET_LOWER_VALUES, map_index);

        let tile_count = match map {
            OutdoorMap::Moon => 158,
            _ => TILES_PER_TILESET,
        };

        let tiles = (0..TILES_PER_TILESET)
            .map(|i| {
                let mut pixels = Vec::with_capacity(PIXELS_PER_TILE);

                if i < tile_count {
                    for j in 0..(PIXELS_PER_TILE / 2) {
                        let lower_value = lower_values[i * 32 + j];
                        pixels.push(upper_values[i] + (lower_value & 0x0F));
                        pixels.push(upper_values[i] + (lower_value >> 4));
                    }
                } else {
                    for _ in 0..64 {
                        pixels.push(0);
                    }
                }

                OutdoorTile { pixels }
            })
            .collect();

        let composition_data = rom.read_bytes(record::OUTDOOR_TILESET_COMPOSITION, map_index);

        let composition = (0..COMPOSED_TILES_PER_TILESET)
            .map(|i| {
                TileComposition {
                    upper_left: composition_data[i] as usize,
                    upper_right: composition_data[COMPOSED_TILES_PER_TILESET + i] as usize,
                    lower_left: composition_data[COMPOSED_TILES_PER_TILESET * 2 + i] as usize,
                    lower_right: composition_data[COMPOSED_TILES_PER_TILESET * 3 + i] as usize,
                }
            })
            .collect();

        OutdoorTileset {
            composition,
            palette: rom.read_palette(record::OUTDOOR_TILESET_PALETTE, map_index, 1),
            tiles,
        }
    }
}
