use image::Rgba;

use crate::graphics;
use crate::rom;
use crate::rom_map::record;

const TILEMAP_FLAGS_HIGH_VALUE: u8 = 0x03;
const TILEMAP_FLAGS_PALETTE: u8 = 0x1C;
const TILEMAP_FLAGS_PRIORITY: u8 = 0x20;
const TILEMAP_FLAGS_VERTICAL_FLIP: u8 = 0x40;
const TILEMAP_FLAGS_HORIZONTAL_FLIP: u8 = 0x80;

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
        let tile_count = tile_data.len() / graphics::BYTES_PER_TILE_4BPP;

        let tiles = (0..tile_count)
            .map(|i| {
                graphics::parse_tile_4bpp(
                    &tile_data[i * graphics::BYTES_PER_TILE_4BPP
                        ..(i + 1) * graphics::BYTES_PER_TILE_4BPP],
                )
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
