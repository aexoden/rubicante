use ff4::rom;

use crate::config;
use crate::input;
use crate::map;
use crate::util;

pub struct World {
    pub config: config::Config,
    pub input: input::State,
    pub rom: rom::Rom,

    pub map_index: map::OutdoorMap,

    pub player_position: util::Position,
    pub player_movement: util::Movement,
}

impl World {
    pub fn new(config: config::Config, rom: rom::Rom) -> Self {
        Self {
            config,
            input: input::State::new(),
            rom,
            map_index: map::OutdoorMap::Overworld,
            player_position: util::Position { x: 102, y: 158 },
            player_movement: util::Movement::None,
        }
    }
}
