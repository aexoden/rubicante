use image::Rgba;

use crate::rom;
use crate::rom_map::record;

pub const PIXELS_PER_TILE: usize = 64;
pub const COMPOSED_TILES_PER_TILESET: usize = 128;
pub const TILES_PER_TILESET: usize = 256;

#[derive(Copy, Clone)]
pub struct OutdoorTileProperties {
    pub can_walk_low: bool,
    yellow_chocobo: bool,
    black_chocobo: bool,
    pub forest: bool,
    hovercraft: bool,
    airship: bool,
    can_walk_high: bool,
    big_whale: bool,
    hide_lower: bool,
    can_land_airship: bool,
    encounters: bool,
    trigger: bool,
    battle_background: usize,
}

impl OutdoorTileProperties {
    pub fn new(data: &[u8]) -> Self {
        let can_walk_low = data[0] & 0x01 > 0;
        let yellow_chocobo = data[0] & 0x02 > 0;
        let black_chocobo = data[0] & 0x04 > 0;
        let forest = data[0] & 0x08 > 0;
        let hovercraft = data[0] & 0x10 > 0;
        let airship = data[0] & 0x20 > 0;
        let can_walk_high = data[0] & 0x40 > 0;
        let big_whale = data[0] & 0x80 > 0;

        let battle_background = usize::from(data[1] & 0x07);
        let hide_lower = data[1] & 0x08 > 0;
        let can_land_airship = data[1] & 0x10 > 0;
        let encounters = data[1] & 0x40 > 0;
        let trigger = data[1] & 0x80 > 0;

        Self {
            can_walk_low,
            yellow_chocobo,
            black_chocobo,
            forest,
            hovercraft,
            airship,
            can_walk_high,
            big_whale,
            hide_lower,
            can_land_airship,
            encounters,
            trigger,
            battle_background,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum OutdoorMap {
    Overworld,
    Underworld,
    Moon,
}

pub struct Map {
    pub height: usize,
    pub width: usize,
    pub tilemap: Vec<u8>,
}

impl Map {
    pub fn new_outdoor(rom: &rom::Rom, map: OutdoorMap) -> Map {
        let height = match map {
            OutdoorMap::Moon => 64,
            _ => 256,
        };

        let width = height;

        let tilemap_record = match map {
            OutdoorMap::Overworld => record::OUTDOOR_TILEMAP_OVERWORLD,
            OutdoorMap::Underworld => record::OUTDOOR_TILEMAP_UNDERWORLD,
            OutdoorMap::Moon => record::OUTDOOR_TILEMAP_MOON,
        };

        let encoded_tilemap = rom.read_bytes(tilemap_record, 0);
        let mut tilemap = Vec::with_capacity(width * height);

        let mut index = 0;

        while tilemap.len() < width * height {
            match encoded_tilemap[index] {
                0x00 | 0x10 | 0x20 | 0x30 => {
                    tilemap.push(encoded_tilemap[index]);

                    if let OutdoorMap::Overworld = map {
                        tilemap.push(encoded_tilemap[index] / 16 * 3 + 0x70);
                        tilemap.push(encoded_tilemap[index] / 16 * 3 + 0x71);
                        tilemap.push(encoded_tilemap[index] / 16 * 3 + 0x72);
                    }
                }
                0xFF => {}
                x if x < 0x80 => {
                    tilemap.push(x);
                }
                x => {
                    index += 1;

                    for _ in 0..(encoded_tilemap[index] as usize) + 1 {
                        tilemap.push(x & 0x7F);
                    }
                }
            }

            index += 1;
        }

        Map {
            height,
            width,
            tilemap,
        }
    }
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
    pub properties: Vec<OutdoorTileProperties>,
}

impl OutdoorTileset {
    pub fn new(rom: &rom::Rom, map: OutdoorMap) -> OutdoorTileset {
        let map_index = map as usize;

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
            .map(|i| TileComposition {
                upper_left: composition_data[i] as usize,
                upper_right: composition_data[COMPOSED_TILES_PER_TILESET + i] as usize,
                lower_left: composition_data[COMPOSED_TILES_PER_TILESET * 2 + i] as usize,
                lower_right: composition_data[COMPOSED_TILES_PER_TILESET * 3 + i] as usize,
            })
            .collect();

        let properties_data = rom.read_bytes(record::OUTDOOR_TILE_PROPERTIES, map_index);

        let properties = (0..COMPOSED_TILES_PER_TILESET)
            .map(|i| OutdoorTileProperties::new(&properties_data[i * 2..(i + 1) * 2]))
            .collect();

        OutdoorTileset {
            composition,
            palette: rom.read_palette(record::OUTDOOR_TILESET_PALETTE, map_index, 1),
            tiles,
            properties,
        }
    }
}
