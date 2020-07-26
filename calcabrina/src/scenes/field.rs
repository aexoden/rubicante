use std::convert::TryFrom;

use ggez::graphics;
use ggez::{Context, GameResult};
use ggez_goodies::scene;
use ggez_goodies::Point2;
use log::debug;

use ff4::map;
use ff4::misc;

use crate::input;
use crate::scenes;
use crate::world::World;

const FIELD_OF_VIEW: f32 = std::f32::consts::PI / 4.0;

const OCEAN_TILE_INDEX: usize = 0x80;
const TILE_SIZE: usize = 8 * 8;
const NUMBER_OF_TILES: usize = 256;

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

struct TileCache {
    pub tiles: [[u8; TILE_SIZE]; NUMBER_OF_TILES],
}

impl TileCache {
    pub fn new_outdoor(tileset: &map::OutdoorTileset) -> Self {
        let mut tiles = [[0; TILE_SIZE]; NUMBER_OF_TILES];

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

pub struct FieldScene {
    done: bool,
    frame_counter: u32,
    movement_direction: Direction,
    movement_frames: i8,
    theta: f32,
    tile_cache: TileCache,
    transform: Vec<Option<(f32, f32)>>,
    zoom: f32,
}

impl FieldScene {
    pub fn new(_ctx: &mut ggez::Context, world: &mut World) -> Self {
        let (window_width, window_height) = world.config.get_window_size();

        FieldScene {
            done: false,
            frame_counter: 0,
            movement_direction: Direction::Up,
            movement_frames: 0,
            tile_cache: TileCache::new_outdoor(&world.tileset),
            theta: 0.0,
            transform: vec![None; window_width * window_height],
            zoom: 0.0,
        }
    }

    fn animate_ocean_tiles(&mut self, world: &mut World) {
        let (tile_offset, line) = misc::get_ocean_animation_line(
            &world.rom,
            usize::try_from(self.frame_counter >> 1).unwrap(),
        );
        let tile_index = OCEAN_TILE_INDEX + tile_offset;

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

    fn animate_overworld_water_tiles(&mut self, world: &mut World) {
        if self.frame_counter % 2 == 0 {
            self.animate_ocean_tiles(world);
        }
    }

    fn draw_outdoor_map(&mut self, world: &mut World, ctx: &mut Context) -> GameResult {
        let (base_window_width, base_window_height) = world.config.get_base_window_size();
        let (window_width, window_height) = world.config.get_window_size();

        let mut img = vec![0; window_width * window_height * 4];

        let x_scale_factor = window_width as f32 / base_window_width;
        let y_scale_factor = window_height as f32 / base_window_height;

        let focal_distance = (base_window_height / FIELD_OF_VIEW.tan()) / 2.0;

        let (scroll_x, scroll_y) = get_direction_delta(self.movement_direction);

        let scroll_x = match self.movement_frames {
            0 => 0,
            _ => scroll_x * -(16 - i32::try_from(self.movement_frames).unwrap()),
        };

        let scroll_y = match self.movement_frames {
            0 => 0,
            _ => scroll_y * -(16 - i32::try_from(self.movement_frames).unwrap()),
        };

        let center_x = isize::try_from(
            (i32::try_from(world.x).unwrap() * 16 + 8 - scroll_x)
                .rem_euclid(i32::try_from(world.map.width).unwrap() * 16),
        )
        .unwrap();

        let center_y = isize::try_from(
            (i32::try_from(world.y).unwrap() * 16 + 8 - scroll_y)
                .rem_euclid(i32::try_from(world.map.height).unwrap() * 16),
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
                        .rem_euclid(isize::try_from(world.map.width).unwrap() * 16),
                )
                .unwrap();

                let target_y = usize::try_from(
                    (target_y.floor() as isize + center_y)
                        .rem_euclid(isize::try_from(world.map.height).unwrap() * 16),
                )
                .unwrap();

                let tile_index = usize::from(
                    world.map.tilemap[(target_x / 16) + (target_y / 16) * world.map.width],
                );

                let palette_index = self.tile_cache.get_composite_pixel(
                    &world.tileset.composition[tile_index],
                    target_x % 16,
                    target_y % 16,
                );

                let color = world.tileset.palette[usize::from(palette_index)];

                for i in 0..4 {
                    img[index + i] = color[i];
                }
            }
        }

        let img =
            graphics::Image::from_rgba8(ctx, window_width as u16, window_height as u16, &img)?;

        let params = graphics::DrawParam::default().dest(Point2::new(0.0, 0.0));

        graphics::draw(ctx, &img, params)
    }
}

impl scene::Scene<World, input::Event> for FieldScene {
    fn update(&mut self, world: &mut World, ctx: &mut ggez::Context) -> scenes::Switch {
        if self.movement_frames > 0 {
            self.movement_frames -= 1;

            if self.movement_frames == 0 {
                let (delta_x, delta_y) = get_direction_delta(self.movement_direction);

                world.x = u8::try_from(
                    (i32::from(world.x) + delta_x)
                        .rem_euclid(i32::try_from(world.map.width).unwrap()),
                )
                .unwrap();

                world.y = u8::try_from(
                    (i32::from(world.y) + delta_y)
                        .rem_euclid(i32::try_from(world.map.height).unwrap()),
                )
                .unwrap();
            }
        }

        if self.movement_frames == 0 {
            let vertical = world.input.get_axis_raw(input::Axis::Vertical);
            let horizontal = world.input.get_axis_raw(input::Axis::Horizontal);

            if vertical.abs() > 0.5 || horizontal.abs() > 0.5 {
                if horizontal.abs() > 0.5 {
                    self.movement_direction = match horizontal {
                        x if x < -0.5 => Direction::Left,
                        _ => Direction::Right,
                    }
                } else {
                    self.movement_direction = match vertical {
                        y if y < -0.5 => Direction::Up,
                        _ => Direction::Down,
                    }
                }

                self.movement_frames = 16;
            }
        }

        if self.done {
            ggez::event::quit(ctx);
        }

        self.frame_counter += 1;

        scene::SceneSwitch::None
    }

    fn draw(&mut self, world: &mut World, ctx: &mut ggez::Context) -> GameResult {
        let (window_width, window_height) = world.config.get_window_size();

        self.animate_overworld_water_tiles(world);
        self.draw_outdoor_map(world, ctx)?;

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2::new(0.0, 0.0),
            8.0 * world.config.scale,
            0.5,
            graphics::WHITE,
        )?;

        graphics::draw(
            ctx,
            &circle,
            (Point2::new(
                window_width as f32 / 2.0,
                window_height as f32 / 2.0,
            ),),
        )?;

        Ok(())
    }

    fn name(&self) -> &str {
        "FieldScene"
    }

    fn input(&mut self, world: &mut World, event: input::Event, started: bool) {
        debug!("Input: {:?} {}", event, started);

        if started {
            let reset_transform = match event {
                input::Event::Button(input::Button::ZoomIn) => {
                    self.zoom -= 1.0;
                    true
                }
                input::Event::Button(input::Button::ZoomOut) => {
                    self.zoom += 1.0;
                    true
                }
                input::Event::Button(input::Button::RotatePlus) => {
                    self.theta += std::f32::consts::PI / 64.0;
                    true
                }
                input::Event::Button(input::Button::RotateMinus) => {
                    self.theta -= std::f32::consts::PI / 64.0;
                    true
                }
                input::Event::Button(input::Button::Quit) => {
                    self.done = true;
                    false
                }
                _ => false,
            };

            if reset_transform {
                debug!("Zoom: {}, Theta: {}", self.zoom, self.theta);
                let (window_width, window_height) = world.config.get_window_size();
                self.transform = vec![None; window_width * window_height];
            }
        }
    }
}

fn get_direction_delta(direction: Direction) -> (i32, i32) {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}
