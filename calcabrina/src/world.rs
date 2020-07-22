use ff4::map;
use ff4::rom;

use crate::input;

pub struct World {
    pub input: input::State,
    pub x: u8,
    pub y: u8,
    pub map: map::Map,
    pub tileset: map::OutdoorTileset,
}

impl World {
    pub fn new(rom: &rom::Rom) -> Self {
        let map = map::Map::new_outdoor(&rom, map::OutdoorMap::Overworld);
        let tileset = map::OutdoorTileset::new(&rom, map::OutdoorMap::Overworld);

        Self {
            input: input::State::new(),
            x: 36,
            y: 83,
            map,
            tileset,
        }
    }
}
