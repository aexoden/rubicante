use ff4::map;
use ff4::rom;

use crate::config;
use crate::input;
use crate::util;

pub struct World {
    pub config: config::Config,
    pub input: input::State,
    pub rom: rom::Rom,

    pub map: map::Map,
    pub tileset: map::OutdoorTileset,

    pub player_position: util::Position,
}

impl World {
    pub fn new(config: config::Config, rom: rom::Rom) -> Self {
        let map = map::Map::new_outdoor(&rom, map::OutdoorMap::Overworld);
        let tileset = map::OutdoorTileset::new(&rom, map::OutdoorMap::Overworld);

        Self {
            config,
            input: input::State::new(),
            rom,
            map,
            tileset,
            player_position: util::Position { x: 102, y: 158 },
        }
    }
}
