use std::convert::TryFrom;

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::input;
use ggez::timer;
use ggez::{Context, GameResult};
use ggez_goodies::Point2;

use ff4::map;
use ff4::rom;

const BASE_WINDOW_WIDTH: usize = 256;
const BASE_WINDOW_HEIGHT: usize = 224;

const WINDOW_WIDTH: usize = 896;
const WINDOW_HEIGHT: usize = 672;

const TILE_ARRAY_SIZE: usize = 16 * 16;

const FIELD_OF_VIEW: f32 = std::f32::consts::PI / 4.0;

pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let filename = match args.next() {
            Some(arg) => arg,
            None => return Err("No filename given"),
        };

        Ok(Config { filename })
    }
}

#[derive(Copy, Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

pub fn run(rom: &rom::Rom) -> GameResult {
    let (ctx, event_loop) =
        &mut ggez::ContextBuilder::new("calcabrina", "Jason Lynch <jason@calindora.com>")
            .window_setup(conf::WindowSetup::default().title("Calcabrina"))
            .window_mode(
                conf::WindowMode::default().dimensions(WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32),
            )
            .build()?;

    let state = &mut MainState::new(&rom);

    event::run(ctx, event_loop, state)
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

struct MainState {
    x: u8,
    y: u8,
    map: map::Map,
    movement_direction: Direction,
    movement_frames: i8,
    tileset: map::OutdoorTileset,
    tile_cache: TileCache,
    zoom: f32,
    theta: f32,
    transform: Vec<Option<(f32, f32)>>,
}

impl MainState {
    fn new(rom: &rom::Rom) -> Self {
        let map = map::Map::new_outdoor(&rom, map::OutdoorMap::Overworld);
        let tileset = map::OutdoorTileset::new(&rom, map::OutdoorMap::Overworld);
        let tile_cache = TileCache::new_outdoor(&tileset);

        Self {
            x: 36,
            y: 83,
            map,
            movement_direction: Direction::Up,
            movement_frames: 0,
            tileset,
            tile_cache,
            zoom: 0.0,
            theta: 0.0,
            transform: vec![None; WINDOW_HEIGHT * WINDOW_WIDTH],
        }
    }

    fn draw_outdoor_map(&mut self, ctx: &mut Context) -> GameResult {
        let mut img = vec![0; WINDOW_WIDTH * WINDOW_HEIGHT * 4];

        let x_scale_factor = WINDOW_WIDTH as f32 / BASE_WINDOW_WIDTH as f32;
        let y_scale_factor = WINDOW_HEIGHT as f32 / BASE_WINDOW_HEIGHT as f32;

        let focal_distance = (BASE_WINDOW_HEIGHT as f32 / FIELD_OF_VIEW.tan()) / 2.0;

        let (scroll_x, scroll_y) = get_direction_delta(self.movement_direction);

        let scroll_x = match self.movement_frames {
            0 => 0,
            _ => scroll_x * -(16 - isize::try_from(self.movement_frames).unwrap()),
        };

        let scroll_y = match self.movement_frames {
            0 => 0,
            _ => scroll_y * -(16 - isize::try_from(self.movement_frames).unwrap()),
        };

        let center_x = isize::try_from(
            (isize::try_from(self.x).unwrap() * 16 + 8 - scroll_x)
                .rem_euclid(isize::try_from(self.map.width).unwrap() * 16),
        )
        .unwrap();

        let center_y = isize::try_from(
            (isize::try_from(self.y).unwrap() * 16 + 8 - scroll_y)
                .rem_euclid(isize::try_from(self.map.height).unwrap() * 16),
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

                let palette_index =
                    self.tile_cache.tiles[tile_index][target_x % 16 + target_y % 16 * 16];

                let color = self.tileset.palette[usize::from(palette_index)];

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

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, 60) {
            if self.movement_frames > 0 {
                self.movement_frames -= 1;

                if self.movement_frames == 0 {
                    let (delta_x, delta_y) = match self.movement_direction {
                        Direction::Up => (0, -1),
                        Direction::Down => (0, 1),
                        Direction::Left => (-1, 0),
                        Direction::Right => (1, 0),
                    };

                    self.x = u8::try_from(
                        (i32::from(self.x) + delta_x)
                            .rem_euclid(i32::try_from(self.map.width).unwrap()),
                    )
                    .unwrap();
                    self.y = u8::try_from(
                        (i32::from(self.y) + delta_y)
                            .rem_euclid(i32::try_from(self.map.height).unwrap()),
                    )
                    .unwrap();
                }
            }

            if self.movement_frames == 0 {
                for key in input::keyboard::pressed_keys(ctx) {
                    self.movement_direction = match key {
                        event::KeyCode::Up => Direction::Up,
                        event::KeyCode::Down => Direction::Down,
                        event::KeyCode::Left => Direction::Left,
                        event::KeyCode::Right => Direction::Right,
                        _ => continue,
                    };

                    self.movement_frames = 16;
                }
            }

            for key in input::keyboard::pressed_keys(ctx) {
                match key {
                    event::KeyCode::Q => self.zoom += 1.0,
                    event::KeyCode::W => self.zoom -= 1.0,
                    event::KeyCode::A => self.theta += std::f32::consts::PI / 60.0,
                    event::KeyCode::S => self.theta -= std::f32::consts::PI / 60.0,
                    _ => continue,
                }

                self.transform = vec![None; WINDOW_WIDTH * WINDOW_HEIGHT];
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.0, 0.0, 0.0, 1.0].into());

        self.draw_outdoor_map(ctx)?;

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

        graphics::present(ctx)
    }
}

fn get_direction_delta(direction: Direction) -> (isize, isize) {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}
