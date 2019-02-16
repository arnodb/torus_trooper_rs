use std::collections::HashSet;

use piston::input::keyboard::Key;
use piston::input::{Button, ButtonArgs, ButtonState};
use rle_vec::RleVec;

pub trait Pad {
    fn start_record(&mut self);
    fn start_replay(&mut self, record: RleVec<PadState>);
    fn handle_button_event(&mut self, button_args: &ButtonArgs);
    fn handle_focus_event(&mut self, focus: bool);
    fn get_direction(&self) -> PadDirection;
    fn get_buttons(&self) -> PadButtons;
    fn record(&mut self) -> PadState;
    fn replay(&mut self) -> Option<PadState>;
    fn pause_pressed(&self) -> bool;
    fn esc_pressed(&self) -> bool;
    fn get_record(&mut self) -> RleVec<PadState>;
}

pub struct GamePad {
    button_reversed: bool,
    keys: HashSet<Key>,
    state: PadState,
    record: RleVec<PadState>,
    record_run_index: usize,
    record_run_sub_index: usize,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct PadState {
    pub direction: PadDirection,
    pub buttons: PadButtons,
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
            state: PadState {
                direction: PadDirection::NONE,
                buttons: PadButtons::NONE,
            },
            record: RleVec::new(),
            record_run_index: 0,
            record_run_sub_index: 0,
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
    fn start_record(&mut self) {
        self.record.clear();
        self.record_run_index = 0;
        self.record_run_sub_index = 0;
    }

    fn start_replay(&mut self, record: RleVec<PadState>) {
        self.record = record;
        self.record_run_index = 0;
        self.record_run_sub_index = 0;
    }

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
            self.state = PadState {
                direction: self.calc_direction(),
                buttons: self.calc_buttons(),
            };
        }
    }

    fn handle_focus_event(&mut self, focus: bool) {
        if !focus {
            self.keys.clear();
            self.state = PadState {
                direction: self.calc_direction(),
                buttons: self.calc_buttons(),
            };
        }
    }

    fn get_direction(&self) -> PadDirection {
        self.state.direction
    }

    fn get_buttons(&self) -> PadButtons {
        self.state.buttons
    }

    fn record(&mut self) -> PadState {
        let state = self.state;
        self.record.push(state);
        state
    }

    fn replay(&mut self) -> Option<PadState> {
        let sub_index = self.record_run_sub_index;
        if let Some((state, eor)) = self
            .record
            .runs()
            .nth(self.record_run_index)
            .and_then(|run| {
                if sub_index + 1 < run.len {
                    Some((*run.value, false))
                } else {
                    Some((*run.value, true))
                }
            })
        {
            if !eor {
                self.record_run_sub_index += 1;
            } else {
                self.record_run_index += 1;
                self.record_run_sub_index = 0;
            }
            Some(state)
        } else {
            None
        }
    }

    fn pause_pressed(&self) -> bool {
        self.keys.contains(&Key::P)
    }

    fn esc_pressed(&self) -> bool {
        self.keys.contains(&Key::Escape)
    }

    fn get_record(&mut self) -> RleVec<PadState> {
        std::mem::replace(&mut self.record, RleVec::new())
    }
}
