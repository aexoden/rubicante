use std::process;

use log::error;
use nom::{
    number::complete::{le_u16, le_u24, le_u32, le_u8},
    IResult,
};

use crate::rom;
use crate::rom_map;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Handedness {
    Right,
    Left,
    Both,
}

#[derive(Debug, PartialEq)]
pub struct CharacterInitial {
    pub handedness: Handedness,
    pub id: u8,
    pub long_range: bool,
    pub class: u8,
    pub level: u8,
    pub status: u32,
    pub hp_current: u16,
    pub hp_maximum: u16,
    pub mp_current: u16,
    pub mp_maximum: u16,
    pub strength: u8,
    pub agility: u8,
    pub vitality: u8,
    pub wisdom: u8,
    pub will: u8,
    pub base_critical_rate: u8,
    pub base_critical_bonus: u8,
    pub morale: u8,
    pub experience: u32,
    pub speed_modifier: u8,
    pub current_level_experience: u32,
}

impl CharacterInitial {
    pub fn new(rom: &rom::Rom, index: usize) -> Self {
        let record = rom_map::record::CHARACTER_STATS_INITIAL;
        let bytes = rom.read_bytes(record, index);

        let (_, result) = parse_initial_stats(bytes).unwrap_or_else(|err| {
            error!("Parsing Error: {}", err);
            process::exit(1);
        });

        result
    }
}

pub fn parse_initial_stats(input: &[u8]) -> IResult<&[u8], CharacterInitial> {
    let (input, handedness_id) = le_u8(input)?;
    let handedness = match handedness_id & 0xC0 {
        0x40 => Handedness::Left,
        0x80 => Handedness::Right,
        _ => Handedness::Both,
    };
    let id = handedness_id & 0x3F;

    let (input, long_range_class) = le_u8(input)?;
    let long_range = long_range_class & 0x40 > 0;
    let class = long_range_class & 0x3F;

    let (input, level) = le_u8(input)?;
    let (input, status) = le_u32(input)?;
    let (input, hp_current) = le_u16(input)?;
    let (input, hp_maximum) = le_u16(input)?;
    let (input, mp_current) = le_u16(input)?;
    let (input, mp_maximum) = le_u16(input)?;
    let (input, strength) = le_u8(input)?;
    let (input, agility) = le_u8(input)?;
    let (input, vitality) = le_u8(input)?;
    let (input, wisdom) = le_u8(input)?;
    let (input, will) = le_u8(input)?;
    let (input, base_critical_rate) = le_u8(input)?;
    let (input, base_critical_bonus) = le_u8(input)?;
    let (input, morale) = le_u8(input)?;
    let (input, experience) = le_u24(input)?;
    let (input, _) = le_u8(input)?; // unused
    let (input, speed_modifier) = le_u8(input)?;
    let (input, _) = le_u8(input)?; // unused
    let (input, current_level_experience) = le_u24(input)?;

    Ok((
        input,
        CharacterInitial {
            handedness,
            id,
            long_range,
            class,
            level,
            status,
            hp_current,
            hp_maximum,
            mp_current,
            mp_maximum,
            strength,
            agility,
            vitality,
            wisdom,
            will,
            base_critical_rate,
            base_critical_bonus,
            morale,
            experience,
            speed_modifier,
            current_level_experience,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_initial_stats() {
        assert_eq!(
            parse_initial_stats(&[
                0xCF, 0x43, 0x08, 0x00, 0x00, 0x00, 0x00, 0xB0, 0x04, 0xB0, 0x04, 0xE7, 0x03, 0xE7,
                0x03, 0x10, 0x20, 0x30, 0x40, 0x50, 0x03, 0x28, 0x0A, 0xB8, 0x0B, 0x00, 0x00, 0x10,
                0x00, 0xC4, 0x09, 0x00
            ]),
            Ok((
                &[][..],
                CharacterInitial {
                    handedness: Handedness::Both,
                    id: 15,
                    long_range: true,
                    class: 3,
                    level: 8,
                    status: 0,
                    hp_current: 1200,
                    hp_maximum: 1200,
                    mp_current: 999,
                    mp_maximum: 999,
                    strength: 16,
                    agility: 32,
                    vitality: 48,
                    wisdom: 64,
                    will: 80,
                    base_critical_rate: 3,
                    base_critical_bonus: 40,
                    morale: 10,
                    experience: 3000,
                    speed_modifier: 16,
                    current_level_experience: 2500,
                }
            ))
        );
    }
}
