use crate::rom;
use crate::rom_map;

pub fn get_ocean_animation_line(rom: &rom::Rom, frame: usize) -> (usize, usize) {
    let record = rom_map::record::OCEAN_ANIMATION_SEQUENCE;
    let line = usize::from(rom.read_u8(record.address + frame % 16)) >> 3;

    (line / 16 * 2, line % 16)
}
