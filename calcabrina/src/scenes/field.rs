use std::convert::TryFrom;

use ggez::graphics;
use ggez::{Context, GameResult};
use ggez_goodies::scene;
use ggez_goodies::Point2;
use log::info;

use ff4::map;

use crate::input;
use crate::scenes;
use crate::world::World;

use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;

const BASE_WINDOW_WIDTH: usize = 256;
const BASE_WINDOW_HEIGHT: usize = 224;

const FIELD_OF_VIEW: f32 = std::f32::consts::PI / 4.0;

const TILE_ARRAY_SIZE: usize = 16 * 16;

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

struct TileCache {
    pub tiles: [[u8; TILE_ARRAY_SIZE]; 128],
}

impl TileCache {
    pub fn new_outdoor(tileset: &map::OutdoorTileset) -> Self {
        let mut tiles = [[0; TILE_ARRAY_SIZE]; 128];

        for (index, mut tile) in tiles.iter_mut().enumerate() {
            let composition = &tileset.composition[index];

            render_outdoor_tile(&mut tile, &tileset, composition.upper_left, 0, 0);
            render_outdoor_tile(&mut tile, &tileset, composition.upper_right, 8, 0);
            render_outdoor_tile(&mut tile, &tileset, composition.lower_left, 0, 8);
            render_outdoor_tile(&mut tile, &tileset, composition.lower_right, 8, 8);
        }

        Self { tiles }
    }
}

pub struct FieldScene {
    done: bool,
    movement_direction: Direction,
    movement_frames: i8,
    theta: f32,
    tile_cache: TileCache,
    transform: Vec<Option<(f32, f32)>>,
    zoom: f32,
}

impl FieldScene {
    pub fn new(_ctx: &mut ggez::Context, world: &mut World) -> Self {
        FieldScene {
            done: false,
            movement_direction: Direction::Up,
            movement_frames: 0,
            tile_cache: TileCache::new_outdoor(&world.tileset),
            theta: 0.0,
            transform: vec![None; WINDOW_HEIGHT * WINDOW_WIDTH],
            zoom: 0.0,
        }
    }
}

impl FieldScene {
    fn draw_outdoor_map(&mut self, world: &mut World, ctx: &mut Context) -> GameResult {
        let mut img = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT * 4];

        let x_scale_factor = WINDOW_WIDTH as f32 / BASE_WINDOW_WIDTH as f32;
        let y_scale_factor = WINDOW_HEIGHT as f32 / BASE_WINDOW_HEIGHT as f32;

        let focal_distance = (BASE_WINDOW_HEIGHT as f32 / FIELD_OF_VIEW.tan()) / 2.0;

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

        for window_y in 0..WINDOW_HEIGHT {
            for window_x in 0..WINDOW_WIDTH {
                let index = window_x * 4 + window_y * WINDOW_WIDTH * 4;

                let (target_x, target_y) = match self.transform[window_x + window_y * WINDOW_WIDTH]
                {
                    Some((target_x, target_y)) => (target_x, target_y),
                    None => {
                        let x = ((window_x as f32 + 0.5) / x_scale_factor)
                            - (BASE_WINDOW_WIDTH as f32) / 2.0;

                        let y = ((window_y as f32 + 0.5) / y_scale_factor)
                            - (BASE_WINDOW_HEIGHT as f32) / 2.0;

                        let alpha = (x / focal_distance).atan();
                        let beta = (y / focal_distance).atan();

                        let target_y =
                            beta.sin() * (focal_distance + self.zoom) / (beta - self.theta).cos();

                        let target_x = alpha.sin()
                            * (target_y * (-self.theta).sin() + focal_distance + self.zoom)
                            / alpha.cos();

                        self.transform[window_x + window_y * WINDOW_WIDTH] =
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

                let palette_index =
                    self.tile_cache.tiles[tile_index][target_x % 16 + target_y % 16 * 16];

                let color = world.tileset.palette[usize::from(palette_index)];

                for i in 0..4 {
                    img[index + i] = color[i];
                }
            }
        }

        let img =
            graphics::Image::from_rgba8(ctx, WINDOW_WIDTH as u16, WINDOW_HEIGHT as u16, &img)?;

        let params = graphics::DrawParam::default().dest(Point2::new(0.0, 0.0));

        graphics::draw(ctx, &img, params)
    }
}

impl scene::Scene<World, input::Event> for FieldScene {
    fn update(&mut self, world: &mut World, _ctx: &mut ggez::Context) -> scenes::Switch {
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
            scene::SceneSwitch::Pop
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut World, ctx: &mut ggez::Context) -> GameResult {
        self.draw_outdoor_map(world, ctx)?;

        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            Point2::new(0.0, 0.0),
            17.0,
            0.5,
            graphics::WHITE,
        )?;

        graphics::draw(
            ctx,
            &circle,
            (Point2::new(
                WINDOW_WIDTH as f32 / 2.0,
                WINDOW_HEIGHT as f32 / 2.0,
            ),),
        )?;

        Ok(())
    }

    fn name(&self) -> &str {
        "FieldScene"
    }

    fn input(&mut self, _world: &mut World, event: input::Event, _started: bool) {
        info!("Input: {:?}", event);

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
            self.transform = vec![None; WINDOW_WIDTH * WINDOW_HEIGHT];
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

fn render_outdoor_tile(
    tile: &mut [u8; TILE_ARRAY_SIZE],
    tileset: &map::OutdoorTileset,
    tile_index: usize,
    base_x: usize,
    base_y: usize,
) {
    for (i, color) in tileset.tiles[tile_index].pixels.iter().enumerate() {
        let x = base_x + i % 8;
        let y = base_y + i / 8;
        tile[y * 16 + x] = *color;
    }
}
