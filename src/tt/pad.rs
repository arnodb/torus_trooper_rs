use std::collections::HashSet;

use piston::input::keyboard::Key;
use piston::input::{Button, ButtonArgs, ButtonState};

pub trait Pad {
    fn handle_button_event(&mut self, button_args: &ButtonArgs);
    fn get_direction(&self) -> PadDirection;
    fn get_buttons(&self) -> PadButtons;
    fn pause_pressed(&self) -> bool;
    fn esc_pressed(&self) -> bool;
}

pub struct GamePad {
    button_reversed: bool,
    keys: HashSet<Key>,
    direction: PadDirection,
    buttons: PadButtons,
}

bitflags! {
    pub struct PadDirection: u8 {
        const NONE = 0;

        const UP = 0b00000001;
        const DOWN = 0b00000010;
        const LEFT = 0b00000100;
        const RIGHT = 0b00001000;
    }
}

bitflags! {
    pub struct PadButtons: u8 {
        const NONE = 0;

        const A = 0b00000001;
        const B = 0b00000010;

        const ANY = Self::A.bits | Self::B.bits;
    }
}

impl GamePad {
    pub fn new(button_reversed: bool) -> Self {
        GamePad {
            button_reversed,
            keys: HashSet::new(),
            direction: PadDirection::NONE,
            buttons: PadButtons::NONE,
        }
    }

    fn calc_direction(&self) -> PadDirection {
        let mut direction = PadDirection::NONE;
        if self.keys.contains(&Key::Right)
            || self.keys.contains(&Key::NumPad6)
            || self.keys.contains(&Key::D)
        {
            direction |= PadDirection::RIGHT;
        }
        if self.keys.contains(&Key::Left)
            || self.keys.contains(&Key::NumPad4)
            || self.keys.contains(&Key::A)
        {
            direction |= PadDirection::LEFT;
        }
        if self.keys.contains(&Key::Down)
            || self.keys.contains(&Key::NumPad2)
            || self.keys.contains(&Key::S)
        {
            direction |= PadDirection::DOWN;
        }
        if self.keys.contains(&Key::Up)
            || self.keys.contains(&Key::NumPad8)
            || self.keys.contains(&Key::D)
        {
            direction |= PadDirection::UP;
        }
        direction
    }

    fn calc_buttons(&self) -> PadButtons {
        let mut buttons = PadButtons::NONE;
        if self.keys.contains(&Key::Z)
            || self.keys.contains(&Key::Period)
            || self.keys.contains(&Key::LCtrl)
        {
            buttons |= match self.button_reversed {
                false => PadButtons::A,
                true => PadButtons::B,
            };
        }
        if self.keys.contains(&Key::X)
            || self.keys.contains(&Key::Slash)
            || self.keys.contains(&Key::LAlt)
            || self.keys.contains(&Key::LShift)
        {
            buttons |= match self.button_reversed {
                false => PadButtons::B,
                true => PadButtons::A,
            };
        }
        buttons
    }
}

impl Pad for GamePad {
    fn handle_button_event(&mut self, button_args: &ButtonArgs) {
        if let Button::Keyboard(key) = button_args.button {
            match button_args.state {
                ButtonState::Press => {
                    self.keys.insert(key);
                }
                ButtonState::Release => {
                    self.keys.remove(&key);
                }
            }
            self.direction = self.calc_direction();
            self.buttons = self.calc_buttons();
        }
    }

    fn get_direction(&self) -> PadDirection {
        self.direction
    }

    fn get_buttons(&self) -> PadButtons {
        self.buttons
    }

    fn pause_pressed(&self) -> bool {
        self.keys.contains(&Key::P)
    }

    fn esc_pressed(&self) -> bool {
        self.keys.contains(&Key::Escape)
    }
}
