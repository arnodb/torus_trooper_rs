#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

#[macro_use]
mod macros;

pub mod tt;
pub mod util;

pub mod gl;
pub mod glu;

#[cfg(feature = "game_recorder")]
mod game_recorder;

use failure::Backtrace;
use piston::event_loop::*;
use piston::input::*;
use std::time::Instant;

use crate::tt::actor::bullet::BulletPool;
use crate::tt::actor::enemy::EnemyPool;
use crate::tt::actor::float_letter::FloatLetterPool;
use crate::tt::actor::particle::ParticlePool;
use crate::tt::actor::shot::ShotPool;
use crate::tt::barrage::BarrageManager;
use crate::tt::camera::Camera;
use crate::tt::errors::GameError;
use crate::tt::letter::Letter;
use crate::tt::manager::stage::StageManager;
use crate::tt::manager::{GameManager, Manager, MoveAction};
use crate::tt::pad::GamePad;
use crate::tt::prefs::PrefManager;
use crate::tt::screen::Screen;
use crate::tt::ship::Ship;
use crate::tt::state::shared::SharedState;
use crate::tt::tunnel::{Torus, Tunnel};
use crate::tt::ActionParams;
use crate::util::rand::Rand;

struct MainLoop {
    done: bool,
}

impl MainLoop {
    fn new() -> Self {
        MainLoop { done: false }
    }

    fn main(&mut self) -> Result<(), GameError> {
        let mut pref_manager = PrefManager::new();

        let mut screen = Screen::new();
        screen.init_opengl()?;

        let mut pad = GamePad::new(false);

        let letter = Letter::new(&screen);

        let initial_seed = Rand::rand_seed();

        let mut tunnel = Tunnel::new(Torus::new(initial_seed));

        let mut camera = Camera::new();
        let mut ship = Ship::new(&screen, initial_seed);

        let mut barrage_manager = BarrageManager::load(&screen)?;
        let mut shots = ShotPool::new(64, &screen);
        let mut bullets = BulletPool::new(512, initial_seed);
        let mut enemies = EnemyPool::new(64, initial_seed, &screen);
        let mut particles = ParticlePool::new(1024, initial_seed);
        let mut float_letters = FloatLetterPool::new(16);

        let mut stage_manager = StageManager::new(initial_seed);

        let mut manager = GameManager::new(&screen)?;
        let mut shared_state = SharedState::new();

        let mut events = Events::new(EventSettings::new().swap_buffers(true));

        let mut params = ActionParams {
            pref_manager: &mut pref_manager,
            screen: &mut screen,
            letter: &letter,
            pad: &mut pad,
            shared_state: &mut shared_state,
            stage_manager: &mut stage_manager,
            camera: &mut camera,
            ship: &mut ship,
            tunnel: &mut tunnel,
            barrage_manager: &mut barrage_manager,
            shots: &mut shots,
            bullets: &mut bullets,
            enemies: &mut enemies,
            particles: &mut particles,
            float_letters: &mut float_letters,
            #[cfg(feature = "game_recorder")]
            next_recorder_id: record_next_id!(),
        };

        manager.start(&mut params);

        let start_time = Instant::now();
        let mut prev_millis = 0;

        while let Some(e) = events.next(
            params
                .screen
                .window_mut()
                .ok_or_else(|| GameError::Fatal("No window".to_string(), Backtrace::new()))?,
        ) {
            let now_millis = {
                let duration = Instant::now().duration_since(start_time);
                duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
            };
            let mut frame = (now_millis - prev_millis) / 16;
            /*if frame <= 0 {
                frame = 1;
                prev_millis = now_millis;
            } else */
            if frame > 5 {
                frame = 5;
                prev_millis = now_millis
            } else {
                prev_millis += frame * 16;
            }

            for _i in 0..frame {
                let action = manager.mov(&mut params);
                match action {
                    MoveAction::StartTitle(from_game_over) => {
                        record_event_end!(from_game_over);
                        record_stop!();
                        manager.start_title(&mut params, false, from_game_over);
                    }
                    MoveAction::StartInGame => {
                        let new_seed = Rand::rand_seed();
                        manager.start_in_game(new_seed, &mut params);
                    }
                    MoveAction::BreakLoop => self.done = true,
                    MoveAction::None => (),
                }
            }

            if let Some(r) = e.resize_args() {
                params.screen.resized(r);
            }

            if let Some(r) = e.render_args() {
                manager.draw(&mut params, &r);
            }

            if let Some(b) = e.button_args() {
                params.pad.handle_button_event(&b);
            }

            if let Some(f) = e.focus_args() {
                params.pad.handle_focus_event(f);
            }

            if self.done {
                break;
            }
        }

        manager.quit_last(params.pref_manager)
    }
}

fn main() {
    MainLoop::new().main().unwrap();
}
