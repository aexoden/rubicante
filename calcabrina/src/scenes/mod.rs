use ggez_goodies::scene;

use crate::input;
use crate::world::World;

pub mod field;
pub mod title;

pub type Switch = scene::SceneSwitch<World, input::Event>;
pub type Stack = scene::SceneStack<World, input::Event>;
