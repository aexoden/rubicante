use std::convert::TryFrom;

use ggez::graphics;
use ggez::{Context, GameResult};
use ggez_goodies::scene;
use ggez_goodies::Point2;
use log::debug;

use crate::input;
use crate::scenes;
use crate::util;
use crate::util::{Direction, Movement};
use crate::world::World;

pub mod map;
pub mod sprite;

pub struct FieldScene {
    done: bool,
    map: map::Map,
    player_sprite: sprite::FieldSprite,
}

impl FieldScene {
    pub fn new(_ctx: &mut Context, world: &mut World) -> Self {
        FieldScene {
            done: false,
            map: map::Map::new_outdoor(&world.config, &world.rom, world.map_index),
            player_sprite: sprite::FieldSprite::new_player(&world.rom, 0),
        }
    }

    pub fn do_player_movement(&mut self, world: &mut World) {
        if let Movement::Direction {
            direction,
            ref mut frame_counter,
        } = world.player_movement
        {
            *frame_counter -= 1;

            if *frame_counter == 0 {
                let (delta_x, delta_y) = util::get_direction_delta(direction);

                world.player_position.x = u8::try_from(
                    (i32::from(world.player_position.x) + delta_x)
                        .rem_euclid(i32::try_from(self.map.width()).unwrap()),
                )
                .unwrap();

                world.player_position.y = u8::try_from(
                    (i32::from(world.player_position.y) + delta_y)
                        .rem_euclid(i32::try_from(self.map.height()).unwrap()),
                )
                .unwrap();

                world.player_movement = Movement::None;
            }
        }

        if let Movement::None = world.player_movement {
            let vertical = world.input.get_axis_raw(input::Axis::Vertical);
            let horizontal = world.input.get_axis_raw(input::Axis::Horizontal);

            if vertical.abs() > 0.5 || horizontal.abs() > 0.5 {
                world.player_movement = Movement::Direction {
                    direction: if horizontal.abs() > 0.5 {
                        match horizontal {
                            x if x < -0.5 => Direction::Left,
                            _ => Direction::Right,
                        }
                    } else {
                        match vertical {
                            y if y < -0.5 => Direction::Up,
                            _ => Direction::Down,
                        }
                    },
                    frame_counter: 16,
                };

                if let Movement::Direction {
                    direction,
                    frame_counter: _,
                } = world.player_movement
                {
                    world.player_pose = sprite::Pose::Direction(direction);
                }
            }
        }
    }
}

impl scene::Scene<World, input::Event> for FieldScene {
    fn update(&mut self, world: &mut World, ctx: &mut Context) -> scenes::Switch {
        if self.done {
            ggez::event::quit(ctx);
        }

        if self.map.index() != world.map_index {
            self.map = map::Map::new_outdoor(&world.config, &world.rom, world.map_index);
        }

        while world.party[world.player_sprite_index].is_none() {
            world.player_sprite_index = (world.player_sprite_index + 1) % 5;
        }

        if let Some(character) = &world.party[world.player_sprite_index] {
            if self.player_sprite.class != usize::from(character.class) {
                self.player_sprite =
                    sprite::FieldSprite::new_player(&world.rom, usize::from(character.class));
            }
        }

        self.do_player_movement(world);
        self.map.update(world);

        scene::SceneSwitch::None
    }

    fn draw(&mut self, world: &mut World, ctx: &mut Context) -> GameResult {
        let map_img = self.map.render(world, ctx)?;
        let params = graphics::DrawParam::default().dest(Point2::new(0.0, 0.0));
        graphics::draw(ctx, &map_img, params)?;

        let player_sprite_img = self.player_sprite.render(world, ctx)?;
        let coordinates = self.player_sprite.get_draw_coordinates(world);
        let params = graphics::DrawParam::default()
            .dest(coordinates)
            .scale(world.config.get_scale_vector());
        graphics::draw(ctx, &player_sprite_img, params)?;

        Ok(())
    }

    fn name(&self) -> &str {
        "FieldScene"
    }

    fn input(&mut self, _world: &mut World, event: input::Event, started: bool) {
        debug!("Input: {:?} {}", event, started);

        if started {
            if let input::Event::Button(input::Button::Quit) = event {
                self.done = true;
            }

            if let input::Event::Button(input::Button::ChangeMap) = event {
                _world.map_index = match _world.map_index {
                    map::OutdoorMap::Overworld => map::OutdoorMap::Underworld,
                    map::OutdoorMap::Underworld => map::OutdoorMap::Moon,
                    map::OutdoorMap::Moon => map::OutdoorMap::Overworld,
                }
            }
        }
    }
}
