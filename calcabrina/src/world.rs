use ff4::rom;

use crate::character;
use crate::config;
use crate::input;
use crate::scenes;
use crate::scenes::field::map;
use crate::util;

pub struct World {
    pub config: config::Config,
    pub input: input::State,
    pub rom: rom::Rom,

    pub map_index: map::OutdoorMap,

    pub player_position: util::Position,
    pub player_pose: scenes::field::sprite::Pose,
    pub player_movement: util::Movement,
    pub player_sprite_index: usize,

    pub party: [Option<character::Character>; 5],
}

impl World {
    pub fn new(config: config::Config, rom: rom::Rom) -> Self {
        let party = [
            Some(character::Character::new(&rom, 0)),
            None,
            None,
            None,
            None,
        ];

        Self {
            config,
            input: input::State::new(),
            rom,
            map_index: map::OutdoorMap::Overworld,
            player_position: util::Position { x: 102, y: 158 },
            player_pose: scenes::field::sprite::Pose::Direction(util::Direction::Down),
            player_movement: util::Movement::None,
            player_sprite_index: 0,
            party,
        }
    }
}
