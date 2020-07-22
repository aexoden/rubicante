use ff4::map;
use ff4::rom;

use crate::config;
use crate::input;

pub struct World {
    pub config: config::Config,
    pub input: input::State,
    pub x: u8,
    pub y: u8,
    pub map: map::Map,
    pub tileset: map::OutdoorTileset,
}

impl World {
    pub fn new(config: config::Config, rom: &rom::Rom) -> Self {
        let map = map::Map::new_outdoor(&rom, map::OutdoorMap::Overworld);
        let tileset = map::OutdoorTileset::new(&rom, map::OutdoorMap::Overworld);

        Self {
            config,
            input: input::State::new(),
            x: 36,
            y: 83,
            map,
            tileset,
        }
    }
}
