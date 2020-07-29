use ff4::character;
use ff4::rom;

pub struct Character {
    pub class: u8,
}

impl Character {
    pub fn new(rom: &rom::Rom, index: usize) -> Self {
        let initial_stats = character::CharacterInitial::new(rom, index);

        Character {
            class: initial_stats.class,
        }
    }
}
