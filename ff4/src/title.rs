use image::Rgba;

use super::rom;
use super::rom_map::record;

const TILEMAP_FLAGS_HIGH_VALUE: u8 = 0x03;
const TILEMAP_FLAGS_PALETTE: u8 = 0x1C;
const TILEMAP_FLAGS_PRIORITY: u8 = 0x20;
const TILEMAP_FLAGS_VERTICAL_FLIP: u8 = 0x40;
const TILEMAP_FLAGS_HORIZONTAL_FLIP: u8 = 0x80;

const BYTES_PER_TILE_4BPP: usize = 8 * 8 / 2;
const BYTES_PER_TILE_UNPACKED: usize = 8 * 8;

pub struct TilemapEntry {
    pub hflip: bool,
    pub vflip: bool,
    pub priority: bool,
    pub palette: u8,
    pub value: u16,
}

pub struct Title {
    pub palette: Vec<Rgba<u8>>,
    pub tiles: Vec<Vec<u8>>,
    pub tilemap: Vec<TilemapEntry>,
    pub width: usize,
    pub height: usize,
}

impl Title {
    pub fn new(rom: &rom::Rom) -> Self {
        let tilemap = parse_tilemap(rom.read_bytes(record::TITLE_TILEMAP, 0));
        let palette = rom.read_palette(record::TITLE_PALETTE, 0, 1);

        let tile_data = rom.read_bytes(record::TITLE_TILES, 0);
        let tile_count = tile_data.len() / BYTES_PER_TILE_4BPP;

        let tiles = (0..tile_count)
            .map(|i| {
                parse_tile_4bpp(&tile_data[i * BYTES_PER_TILE_4BPP..(i + 1) * BYTES_PER_TILE_4BPP])
            })
            .collect();

        Title {
            palette,
            tiles,
            tilemap,
            width: 256,
            height: 256,
        }
    }
}

fn parse_tile_4bpp(data: &[u8]) -> Vec<u8> {
    (0..BYTES_PER_TILE_UNPACKED)
        .map(|i| {
            let shift = 7 - (i % 8);
            let row = i / 8;

            let plane_0_index = row * 2;
            let plane_1_index = row * 2 + 1;
            let plane_2_index = 16 + row * 2;
            let plane_3_index = 16 + row * 2 + 1;

            ((data[plane_0_index] >> shift) & 0x01)
                + (((data[plane_1_index] >> shift) & 0x01) << 1)
                + (((data[plane_2_index] >> shift) & 0x01) << 2)
                + (((data[plane_3_index] >> shift) & 0x01) << 3)
        })
        .collect()
}

fn parse_tilemap(data: &[u8]) -> Vec<TilemapEntry> {
    (0..data.len() / 2)
        .map(|i| {
            let flags = data[i * 2 + 1];

            let value =
                data[i * 2] as u16 + ((flags as u16 & TILEMAP_FLAGS_HIGH_VALUE as u16) << 8);
            let palette = (flags & TILEMAP_FLAGS_PALETTE) >> 2;
            let priority = (flags & TILEMAP_FLAGS_PRIORITY) > 0;
            let vflip = (flags & TILEMAP_FLAGS_VERTICAL_FLIP) > 0;
            let hflip = (flags & TILEMAP_FLAGS_HORIZONTAL_FLIP) > 0;

            TilemapEntry {
                hflip,
                vflip,
                priority,
                palette,
                value,
            }
        })
        .collect()
}
