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

const WINDOW_WIDTH: f32 = 256.0;
const WINDOW_HEIGHT: f32 = 224.0;

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
            .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
            .build()?;

    let state = &mut MainState::new(&rom);

    event::run(ctx, event_loop, state)
}

struct MainState {
    x: u8,
    y: u8,
    map: map::Map,
    movement_direction: Direction,
    movement_frames: i8,
    tileset: map::OutdoorTileset,
}

impl MainState {
    fn new(rom: &rom::Rom) -> Self {
        let map = map::Map::new_outdoor(&rom, map::OutdoorMap::Overworld);
        let tileset = map::OutdoorTileset::new(&rom, map::OutdoorMap::Overworld);

        Self {
            x: 36,
            y: 83,
            map,
            movement_direction: Direction::Up,
            movement_frames: 0,
            tileset,
        }
    }

    fn draw_outdoor_map(&mut self, ctx: &mut Context) -> GameResult {
        let (scroll_x, scroll_y) = get_direction_delta(self.movement_direction);

        let scroll_x = match self.movement_frames {
            0 => scroll_x,
            _ => scroll_x * -(16 - isize::try_from(self.movement_frames).unwrap()),
        };

        let scroll_y = match self.movement_frames {
            0 => scroll_y,
            _ => scroll_y * -(16 - isize::try_from(self.movement_frames).unwrap()),
        };

        let mut img = vec![0; 256 * 224 * 4];

        for row in 0..17 {
            for col in 0..19 {
                let tile_x = i32::from((col - 9) + self.x)
                    .rem_euclid(i32::try_from(self.map.width).unwrap());
                let tile_y = i32::from((row - 8) + self.y)
                    .rem_euclid(i32::try_from(self.map.height).unwrap());
                let tile_index = self.map.tilemap[usize::try_from(
                    tile_x + tile_y * i32::try_from(self.map.width).unwrap(),
                )
                .unwrap()];

                draw_outdoor_composed_tile(
                    &mut img,
                    &self.tileset,
                    usize::from(tile_index),
                    (isize::from(col) - 9) * 16 + 120 + scroll_x,
                    (isize::from(row) - 8) * 16 + 104 + scroll_y,
                );
            }
        }

        let img = graphics::Image::from_rgba8(ctx, 256, 224, &img)?;

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
            8.0,
            0.5,
            graphics::WHITE,
        )?;

        graphics::draw(ctx, &circle, (Point2::new(128.0, 112.0),))?;

        graphics::present(ctx)
    }
}

fn draw_outdoor_tile(
    img: &mut Vec<u8>,
    tileset: &map::OutdoorTileset,
    index: usize,
    base_x: isize,
    base_y: isize,
) {
    for (i, palette_index) in tileset.tiles[index].pixels.iter().enumerate() {
        let x = base_x + isize::try_from(i).unwrap() % 8;
        let y = base_y + isize::try_from(i).unwrap() / 8;
        let color = tileset.palette[usize::from(*palette_index)];

        if x > 0 && x < 256 && y > 0 && y < 224 {
            for i in 0..4 {
                img[usize::try_from(x * 4 + y * 256 * 4 + i).unwrap()] =
                    color[usize::try_from(i).unwrap()];
            }
        }
    }
}

fn draw_outdoor_composed_tile(
    mut img: &mut Vec<u8>,
    tileset: &map::OutdoorTileset,
    index: usize,
    base_x: isize,
    base_y: isize,
) {
    let composition = &tileset.composition[index];

    draw_outdoor_tile(&mut img, &tileset, composition.upper_left, base_x, base_y);

    draw_outdoor_tile(
        &mut img,
        &tileset,
        composition.upper_right,
        base_x + 8,
        base_y,
    );

    draw_outdoor_tile(
        &mut img,
        &tileset,
        composition.lower_left,
        base_x,
        base_y + 8,
    );

    draw_outdoor_tile(
        &mut img,
        &tileset,
        composition.lower_right,
        base_x + 8,
        base_y + 8,
    );
}

fn get_direction_delta(direction: Direction) -> (isize, isize) {
    match direction {
        Direction::Up => (0, -1),
        Direction::Down => (0, 1),
        Direction::Left => (-1, 0),
        Direction::Right => (1, 0),
    }
}
