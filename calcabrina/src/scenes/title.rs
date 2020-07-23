use ggez::graphics;
use ggez_goodies::scene;

use ff4::title;

use crate::input;
use crate::scenes;
use crate::world::World;

const MAX_BRIGHTNESS: u32 = 64;

pub struct TitleScene {
    brightness: u32,
    delay: u32,
    done: bool,
    title: title::Title,
}

impl TitleScene {
    pub fn new(_ctx: &mut ggez::Context, world: &mut World) -> Self {
        TitleScene {
            brightness: 0,
            delay: 30,
            done: false,
            title: title::Title::new(&world.rom),
        }
    }
}

impl scene::Scene<World, input::Event> for TitleScene {
    fn update(&mut self, world: &mut World, ctx: &mut ggez::Context) -> scenes::Switch {
        if self.done && self.brightness > 0 {
            self.brightness -= 1;
        } else if self.delay > 0 {
            self.delay -= 1;
        } else if !self.done && self.brightness < MAX_BRIGHTNESS {
            self.brightness += 1;
        }

        if self.done && self.brightness == 0 && self.delay == 0 {
            scene::SceneSwitch::Replace(Box::new(scenes::field::FieldScene::new(ctx, world)))
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult {
        let mut img = vec![0; self.title.width * self.title.height * 4];

        for (i, entry) in self.title.tilemap.iter().enumerate() {
            let base_x = (i % 32) * 8;
            let base_y = (i / 32) * 8;
            let tile = &self.title.tiles[usize::from(entry.value)];

            for (j, value) in tile.iter().enumerate() {
                let x = base_x + j % 8;
                let y = base_y + j / 8;
                let index = (x + y * self.title.width) * 4;
                let palette_index = entry.palette * 16 + value;

                let color = self.title.palette[usize::from(palette_index)];

                for k in 0..3 {
                    img[index + k] = match self.brightness {
                        MAX_BRIGHTNESS => color[k],
                        _ => {
                            (((color[k] as f32) * (self.brightness as f32)
                                / (MAX_BRIGHTNESS as f32))
                                .round()) as u8
                        }
                    }
                }

                img[index + 3] = if palette_index % 16 == 0 { 0 } else { color[3] }
            }
        }

        let mut img = graphics::Image::from_rgba8(
            ctx,
            self.title.width as u16,
            self.title.height as u16,
            &img,
        )?;
        img.set_filter(graphics::FilterMode::Nearest);

        let params = graphics::DrawParam::default()
            .dest(world.config.get_standard_offset())
            .scale(world.config.get_scale_vector());

        graphics::draw(ctx, &img, params)
    }

    fn name(&self) -> &str {
        "TitleScene"
    }

    fn input(&mut self, _world: &mut World, event: input::Event, _started: bool) {
        if let input::Event::Button(input::Button::Confirm) = event {
            if self.brightness == MAX_BRIGHTNESS {
                self.done = true;
                self.delay = 30;
            }
        }
    }
}
