use image::Rgba;

use crate::rom;
use crate::rom_map;

pub const BYTES_PER_TILE_3BPP: usize = 8 * 8 * 3 / 8;
pub const BYTES_PER_TILE_4BPP: usize = 8 * 8 / 2;
pub const BYTES_PER_TILE_UNPACKED: usize = 8 * 8;

pub const FIELD_SPRITE_PLAYER_FRAME_COUNT: usize = 16;

#[derive(Copy, Clone, Debug)]
pub struct TileReference {
    pub tile: usize,
    pub vflip: bool,
    pub hflip: bool,
}

impl TileReference {
    fn new(data: &[u8]) -> Self {
        Self {
            tile: usize::from(data[0]),
            vflip: data[1] & 0x80 > 0,
            hflip: data[1] & 0x40 > 0,
        }
    }
}

#[derive(Debug)]
pub struct SpriteComposition {
    pub upper_left: TileReference,
    pub upper_right: TileReference,
    pub lower_left: TileReference,
    pub lower_right: TileReference,
}

impl SpriteComposition {
    fn new(data: &[u8]) -> Self {
        Self {
            upper_left: TileReference::new(&data[0..2]),
            upper_right: TileReference::new(&data[2..4]),
            lower_left: TileReference::new(&data[4..6]),
            lower_right: TileReference::new(&data[6..8]),
        }
    }
}

pub struct FieldSpriteSheet {
    pub composition: Vec<SpriteComposition>,
    pub tiles: Vec<Vec<u8>>,
    pub palette_index: usize,
}

impl FieldSpriteSheet {
    pub fn new_player(rom: &rom::Rom, index: usize) -> Self {
        let record = rom_map::record::FIELD_SPRITE_SHEET_PLAYER;
        let bytes = rom.read_bytes(record, index);

        let tiles = (0..bytes.len() / 24)
            .map(|i| parse_tile_3bpp(&bytes[i * 24..(i + 1) * 24]))
            .collect();

        let composition = (0..FIELD_SPRITE_PLAYER_FRAME_COUNT)
            .map(|i| {
                SpriteComposition::new(
                    rom.read_bytes(rom_map::record::FIELD_SPRITE_COMPOSITION_PLAYER, i),
                )
            })
            .collect();

        let record = rom_map::record::FIELD_SPRITE_PALETTE_INDEX_PLAYER;
        let palette_index = usize::from(rom.read_u8(record.address + index));

        Self {
            composition,
            tiles,
            palette_index,
        }
    }
}

pub fn get_field_sprite_palette_player(rom: &rom::Rom, index: usize) -> Vec<Rgba<u8>> {
    rom.read_palette(rom_map::record::FIELD_SPRITE_PALETTE_PLAYER, index, 1)
}

pub fn parse_tile_3bpp(data: &[u8]) -> Vec<u8> {
    (0..BYTES_PER_TILE_UNPACKED)
        .map(|i| {
            let shift = 7 - (i % 8);
            let row = i / 8;

            let plane_0_index = row * 2;
            let plane_1_index = row * 2 + 1;
            let plane_2_index = 16 + row;

            ((data[plane_0_index] >> shift) & 0x01)
                | (((data[plane_1_index] >> shift) & 0x01) << 1)
                | (((data[plane_2_index] >> shift) & 0x01) << 2)
        })
        .collect()
}

pub fn parse_tile_4bpp(data: &[u8]) -> Vec<u8> {
    (0..BYTES_PER_TILE_UNPACKED)
        .map(|i| {
            let shift = 7 - (i % 8);
            let row = i / 8;

            let plane_0_index = row * 2;
            let plane_1_index = row * 2 + 1;
            let plane_2_index = 16 + row * 2;
            let plane_3_index = 16 + row * 2 + 1;

            ((data[plane_0_index] >> shift) & 0x01)
                | (((data[plane_1_index] >> shift) & 0x01) << 1)
                | (((data[plane_2_index] >> shift) & 0x01) << 2)
                | (((data[plane_3_index] >> shift) & 0x01) << 3)
        })
        .collect()
}
