use image::Rgba;

use ggez::graphics;
use ggez::{Context, GameResult};
use ggez_goodies::Point2;

use ff4::rom;

use crate::util;
use crate::util::Direction;
use crate::world::World;

pub enum Pose {
    Direction(Direction),
    Waving,
    RaisedArm,
    LoweredHead,
    Lying,
}

pub struct FieldSprite {
    pub class: usize,
    pub frames: Vec<util::PixelBuffer>,
    pub palette: Vec<Rgba<u8>>,
    pub sheet: ff4::graphics::FieldSpriteSheet,
}

impl FieldSprite {
    pub fn new_player(rom: &rom::Rom, index: usize) -> Self {
        let sheet = ff4::graphics::FieldSpriteSheet::new_player(rom, index);

        let frames = (0..ff4::graphics::FIELD_SPRITE_PLAYER_FRAME_COUNT)
            .map(|frame| {
                let composition = &sheet.composition[frame];
                let mut buffer = util::PixelBuffer::new(16, 16);

                buffer.render_tile(
                    &sheet.tiles[composition.upper_left.tile],
                    composition.upper_left.hflip,
                    composition.upper_left.vflip,
                    0,
                    0,
                );

                buffer.render_tile(
                    &sheet.tiles[composition.upper_right.tile],
                    composition.upper_right.hflip,
                    composition.upper_right.vflip,
                    8,
                    0,
                );

                buffer.render_tile(
                    &sheet.tiles[composition.lower_left.tile],
                    composition.lower_left.hflip,
                    composition.lower_left.vflip,
                    0,
                    8,
                );

                buffer.render_tile(
                    &sheet.tiles[composition.lower_right.tile],
                    composition.lower_right.hflip,
                    composition.lower_right.vflip,
                    8,
                    8,
                );

                buffer
            })
            .collect();

        let palette = ff4::graphics::get_field_sprite_palette_player(rom, sheet.palette_index);

        Self {
            class: index,
            frames,
            palette,
            sheet,
        }
    }

    pub fn get_draw_coordinates(&self, world: &World) -> Point2 {
        let (window_width, window_height) = world.config.get_window_size();

        let mut y_delta = -10.0;

        if let util::Movement::Direction {
            direction,
            frame_counter,
        } = world.player_movement
        {
            if frame_counter <= 8 {
                match direction {
                    Direction::Left | Direction::Right => y_delta -= 1.0,
                    _ => {}
                }
            };
        };

        Point2::new(
            window_width as f32 / 2.0 - 8.0 * world.config.get_scale_vector().x,
            window_height as f32 / 2.0 + y_delta * world.config.get_scale_vector().y,
        )
    }

    pub fn render(
        &self,
        world: &World,
        map: &super::map::Map,
        ctx: &mut Context,
    ) -> GameResult<graphics::Image> {
        let mut frame_index = 2 * match world.player_pose {
            Pose::Direction(Direction::Up) => 0,
            Pose::Direction(Direction::Right) => 1,
            Pose::Direction(Direction::Down) => 2,
            Pose::Direction(Direction::Left) => 3,
            Pose::Waving => 4,
            Pose::RaisedArm => 5,
            Pose::LoweredHead => 6,
            Pose::Lying => 7,
        };

        if let util::Movement::Direction {
            direction: _,
            frame_counter,
        } = world.player_movement
        {
            if frame_counter <= 8 {
                frame_index += 1;
            }
        }

        let tile_properties =
            map.get_tile_properties(world.player_position.x, world.player_position.y, None);

        let frame = &self.frames[frame_index];
        let mut img = vec![0u8; 16 * 16 * 4];

        for (i, palette_index) in frame.pixels.iter().enumerate() {
            let color = self.palette[usize::from(*palette_index)];
            img[i * 4] = color[0];
            img[i * 4 + 1] = color[1];
            img[i * 4 + 2] = color[2];
            img[i * 4 + 3] = if *palette_index == 0 || i >= 128 && tile_properties.forest {
                0
            } else {
                255
            };
        }

        let mut img = graphics::Image::from_rgba8(ctx, 16, 16, &img[..])?;
        img.set_filter(graphics::FilterMode::Nearest);

        Ok(img)
    }
}
