use std::convert::TryFrom;

use ggez::graphics;
use ggez::GameResult;

use ff4::map;
use ff4::misc;
use ff4::rom;

use crate::config;
use crate::util;
use crate::world::World;

pub use map::OutdoorMap;

const FIELD_OF_VIEW: f32 = std::f32::consts::PI / 4.0;

const TILE_INDEX_WATERFALL: usize = 0x7A;
const TILE_INDEX_OCEAN_OVERWORLD: usize = 0x80;
const TILE_INDEX_OCEAN_UNDERWORLD: usize = 0xE0;

pub struct Map {
    frame_counter: usize,
    map: ff4::map::Map,
    index: OutdoorMap,
    zoom: f32,
    theta: f32,
    tile_cache: TileCache,
    tileset: ff4::map::OutdoorTileset,
    transform: Vec<Option<(f32, f32)>>,
}

impl Map {
    pub fn new_outdoor(config: &config::Config, rom: &rom::Rom, index: OutdoorMap) -> Self {
        let (window_width, window_height) = config.get_window_size();
        let map = map::Map::new_outdoor(rom, index);
        let tileset = map::OutdoorTileset::new(rom, index);

        Self {
            frame_counter: 0,
            map,
            index,
            zoom: 0.0,
            theta: 0.0,
            tile_cache: TileCache::new_outdoor(&tileset),
            tileset,
            transform: vec![None; window_width * window_height],
        }
    }

    pub fn update(&mut self, world: &mut World) {
        self.frame_counter += 1;
        self.animate_water_tiles(world);
    }

    pub fn height(&self) -> usize {
        self.map.height
    }

    pub fn index(&self) -> OutdoorMap {
        self.index
    }

    pub fn width(&self) -> usize {
        self.map.width
    }

    pub fn tilemap(&self) -> &Vec<u8> {
        &self.map.tilemap
    }

    pub fn get_tile_properties(
        &self,
        x: u8,
        y: u8,
        direction: Option<util::Direction>,
    ) -> &map::OutdoorTileProperties {
        let x_delta = match direction {
            Some(util::Direction::Up) | Some(util::Direction::Down) | None => 0,
            Some(util::Direction::Right) => 1,
            Some(util::Direction::Left) => -1,
        };

        let y_delta = match direction {
            Some(util::Direction::Left) | Some(util::Direction::Right) | None => 0,
            Some(util::Direction::Up) => -1,
            Some(util::Direction::Down) => 1,
        };

        let x = match self.index {
            OutdoorMap::Overworld | OutdoorMap::Underworld | OutdoorMap::Moon => usize::try_from(
                (i32::from(x) + x_delta).rem_euclid(i32::try_from(self.width()).unwrap()),
            )
            .unwrap(),
        };

        let y = match self.index {
            OutdoorMap::Overworld | OutdoorMap::Underworld | OutdoorMap::Moon => usize::try_from(
                (i32::from(y) + y_delta).rem_euclid(i32::try_from(self.width()).unwrap()),
            )
            .unwrap(),
        };

        let tile_index = usize::from(self.tilemap()[x + y * self.width()]);

        &self.tileset.properties[tile_index]
    }

    pub fn render(
        &mut self,
        world: &World,
        ctx: &mut ggez::Context,
    ) -> GameResult<graphics::Image> {
        let (base_window_width, base_window_height) = world.config.get_base_window_size();
        let (window_width, window_height) = world.config.get_window_size();

        let mut img = vec![0; window_width * window_height * 4];

        let x_scale_factor = window_width as f32 / base_window_width;
        let y_scale_factor = window_height as f32 / base_window_height;

        let focal_distance = (base_window_height / FIELD_OF_VIEW.tan()) / 2.0;

        let (scroll_x, scroll_y) = if let util::Movement::Direction {
            direction,
            frame_counter,
        } = world.player_movement
        {
            let (scroll_x, scroll_y) = util::get_direction_delta(direction);

            let scroll_x = match frame_counter {
                0 => 0,
                _ => scroll_x * -(16 - i32::try_from(frame_counter).unwrap()),
            };

            let scroll_y = match frame_counter {
                0 => 0,
                _ => scroll_y * -(16 - i32::try_from(frame_counter).unwrap()),
            };

            (scroll_x, scroll_y)
        } else {
            (0, 0)
        };

        let center_x = isize::try_from(
            (i32::try_from(world.player_position.x).unwrap() * 16 + 8 - scroll_x)
                .rem_euclid(i32::try_from(self.map.width).unwrap() * 16),
        )
        .unwrap();

        let center_y = isize::try_from(
            (i32::try_from(world.player_position.y).unwrap() * 16 + 8 - scroll_y)
                .rem_euclid(i32::try_from(self.map.height).unwrap() * 16),
        )
        .unwrap();

        for window_y in 0..window_height {
            for window_x in 0..window_width {
                let index = window_x * 4 + window_y * window_width * 4;

                let (target_x, target_y) = match self.transform[window_x + window_y * window_width]
                {
                    Some((target_x, target_y)) => (target_x, target_y),
                    None => {
                        let x =
                            ((window_x as f32 + 0.5) / x_scale_factor) - base_window_width / 2.0;

                        let y =
                            ((window_y as f32 + 0.5) / y_scale_factor) - base_window_height / 2.0;

                        let alpha = (x / focal_distance).atan();
                        let beta = (y / focal_distance).atan();

                        let target_y =
                            beta.sin() * (focal_distance + self.zoom) / (beta - self.theta).cos();

                        let target_x = alpha.sin()
                            * (target_y * (-self.theta).sin() + focal_distance + self.zoom)
                            / alpha.cos();

                        self.transform[window_x + window_y * window_width] =
                            Some((target_x, target_y));

                        (target_x, target_y)
                    }
                };

                let target_x = usize::try_from(
                    (target_x.floor() as isize + center_x)
                        .rem_euclid(isize::try_from(self.map.width).unwrap() * 16),
                )
                .unwrap();

                let target_y = usize::try_from(
                    (target_y.floor() as isize + center_y)
                        .rem_euclid(isize::try_from(self.map.height).unwrap() * 16),
                )
                .unwrap();

                let tile_index = usize::from(
                    self.map.tilemap[(target_x / 16) + (target_y / 16) * self.map.width],
                );

                let palette_index = self.tile_cache.get_composite_pixel(
                    &self.tileset.composition[tile_index],
                    target_x % 16,
                    target_y % 16,
                );

                let color = self.tileset.palette[usize::from(palette_index)];

                for i in 0..4 {
                    img[index + i] = color[i];
                }
            }
        }

        let img =
            graphics::Image::from_rgba8(ctx, window_width as u16, window_height as u16, &img)?;

        Ok(img)
    }

    fn animate_water_tiles(&mut self, world: &mut World) {
        if self.frame_counter % 2 == 0 {
            match self.index {
                OutdoorMap::Overworld | OutdoorMap::Underworld => self.animate_ocean_tiles(world),
                _ => {}
            }
        }

        if let OutdoorMap::Overworld = self.index {
            self.animate_waterfall_tiles(world);
        }
    }

    fn animate_ocean_tiles(&mut self, world: &mut World) {
        let (tile_offset, line) = misc::get_ocean_animation_line(
            &world.rom,
            usize::try_from(self.frame_counter >> 1).unwrap(),
        );

        let tile_index = match self.index {
            OutdoorMap::Overworld => TILE_INDEX_OCEAN_OVERWORLD + tile_offset,
            _ => TILE_INDEX_OCEAN_UNDERWORLD + tile_offset,
        };

        let last_value = self.tile_cache.tiles[tile_index + 1][line * 8 + 7];

        for i in (0..7).rev() {
            self.tile_cache.tiles[tile_index + 1][line * 8 + i + 1] =
                self.tile_cache.tiles[tile_index + 1][line * 8 + i];
        }

        self.tile_cache.tiles[tile_index + 1][line * 8] =
            self.tile_cache.tiles[tile_index][line * 8 + 7];

        for i in (0..7).rev() {
            self.tile_cache.tiles[tile_index][line * 8 + i + 1] =
                self.tile_cache.tiles[tile_index][line * 8 + i];
        }

        self.tile_cache.tiles[tile_index][line * 8] = last_value;
    }

    fn animate_waterfall_tiles(&mut self, world: &mut World) {
        let (tile_offset, column) = misc::get_waterfall_animation_column(
            &world.rom,
            usize::try_from(self.frame_counter).unwrap(),
        );

        let tile_index = TILE_INDEX_WATERFALL + tile_offset;

        for _ in 0..2 {
            let last_value = self.tile_cache.tiles[tile_index + 2][column + 7 * 8];

            for row in (0..7).rev() {
                self.tile_cache.tiles[tile_index + 2][column + (row + 1) * 8] =
                    self.tile_cache.tiles[tile_index + 2][column + row * 8];
            }

            self.tile_cache.tiles[tile_index + 2][column] =
                self.tile_cache.tiles[tile_index][column + 7 * 8];

            for row in (0..7).rev() {
                self.tile_cache.tiles[tile_index][column + (row + 1) * 8] =
                    self.tile_cache.tiles[tile_index][column + row * 8];
            }

            self.tile_cache.tiles[tile_index][column] = last_value;
        }
    }
}

struct TileCache {
    pub tiles: [[u8; ff4::map::PIXELS_PER_TILE]; ff4::map::TILES_PER_TILESET],
}

impl TileCache {
    pub fn new_outdoor(tileset: &map::OutdoorTileset) -> Self {
        let mut tiles = [[0; ff4::map::PIXELS_PER_TILE]; ff4::map::TILES_PER_TILESET];

        for (tile_index, tile) in tiles.iter_mut().enumerate() {
            for (pixel_index, value) in tileset.tiles[tile_index].pixels.iter().enumerate() {
                tile[pixel_index] = *value;
            }
        }

        Self { tiles }
    }

    pub fn get_composite_pixel(
        &self,
        composition: &map::TileComposition,
        x: usize,
        y: usize,
    ) -> u8 {
        let index = if x < 8 && y < 8 {
            composition.upper_left
        } else if x >= 8 && y < 8 {
            composition.upper_right
        } else if x < 8 && y >= 8 {
            composition.lower_left
        } else {
            composition.lower_right
        };

        self.tiles[index][x % 8 + y % 8 * 8]
    }
}
