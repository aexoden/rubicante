use ggez::event::*;
use ggez_goodies::input;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Button {
    ZoomIn,
    ZoomOut,
    RotatePlus,
    RotateMinus,
    Quit,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Axis {
    Vertical,
    Horizontal,
}

pub type Binding = input::InputBinding<Axis, Button>;
pub type Event = input::InputEffect<Axis, Button>;
pub type State = input::InputState<Axis, Button>;

pub fn create_input_binding() -> input::InputBinding<Axis, Button> {
    input::InputBinding::new()
        .bind_key_to_axis(KeyCode::Up, Axis::Vertical, false)
        .bind_key_to_axis(KeyCode::Down, Axis::Vertical, true)
        .bind_key_to_axis(KeyCode::Left, Axis::Horizontal, false)
        .bind_key_to_axis(KeyCode::Right, Axis::Horizontal, true)
        .bind_key_to_button(KeyCode::Q, Button::ZoomIn)
        .bind_key_to_button(KeyCode::W, Button::ZoomOut)
        .bind_key_to_button(KeyCode::A, Button::RotatePlus)
        .bind_key_to_button(KeyCode::S, Button::RotateMinus)
        .bind_key_to_button(KeyCode::Escape, Button::Quit)
}
