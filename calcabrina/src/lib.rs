use std::process;

use ggez::conf;
use ggez::event;
use ggez::graphics;
use ggez::timer;
use ggez::{Context, GameResult};
use log::{error, info};

use ff4::rom;

pub mod config;
pub mod util;

mod input;
mod scenes;
mod world;

pub fn run(config: config::Config) -> GameResult {
    let (window_width, window_height) = config.get_window_size();

    info!(
        "Calculated window size: {} x {}",
        window_width, window_height
    );

    let (ctx, event_loop) =
        &mut ggez::ContextBuilder::new("calcabrina", "Jason Lynch <jason@calindora.com>")
            .window_setup(conf::WindowSetup::default().title("Calcabrina"))
            .window_mode(
                conf::WindowMode::default().dimensions(window_width as f32, window_height as f32),
            )
            .build()?;

    let state = &mut MainState::new(ctx, config);

    event::run(ctx, event_loop, state)
}

struct MainState {
    scenes: scenes::Stack,
    input_binding: input::Binding,
}

impl MainState {
    fn new(ctx: &mut Context, config: config::Config) -> Self {
        let rom = rom::Rom::new(&config.filename).unwrap_or_else(|err| {
            error!("Error loading ROM file: {}", err);
            process::exit(1);
        });

        info!("ROM title: {}", rom.title());
        info!("ROM description: {}", rom.description());

        let world = world::World::new(config, &rom);

        let mut scenes = scenes::Stack::new(ctx, world);

        let scene = Box::new(scenes::field::FieldScene::new(ctx, &mut scenes.world));
        scenes.push(scene);

        Self {
            input_binding: input::create_input_binding(),
            scenes,
        }
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, 60) {
            self.scenes.update(ctx);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::Color::from((0.0, 0.0, 0.0, 0.0)));
        self.scenes.draw(ctx);
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
        _repeat: bool,
    ) {
        if let Some(event) = self.input_binding.resolve(keycode) {
            self.scenes.input(event, true);
            self.scenes.world.input.update_effect(event, true);
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut Context,
        keycode: event::KeyCode,
        _keymod: event::KeyMods,
    ) {
        if let Some(event) = self.input_binding.resolve(keycode) {
            self.scenes.input(event, false);
            self.scenes.world.input.update_effect(event, false);
        }
    }
}
