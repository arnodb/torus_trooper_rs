#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;

pub mod tt;
pub mod util;

pub mod gl;
pub mod glu;

use failure::Backtrace;
use piston::event_loop::*;
use piston::input::*;
use std::time::Instant;

use crate::tt::actor::shot::ShotPool;
use crate::tt::camera::Camera;
use crate::tt::errors::GameError;
use crate::tt::letter::Letter;
use crate::tt::manager::stage::StageManager;
use crate::tt::manager::{GameManager, Manager, MoveAction};
use crate::tt::pad::{GamePad, Pad};
use crate::tt::prefs::PrefManager;
use crate::tt::screen::Screen;
use crate::tt::ship::Ship;
use crate::tt::tunnel::{Torus, Tunnel};
use crate::tt::{DrawParams, MoveParams, StartParams};
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

        let seed = Rand::rand_seed();
        let mut rand = Rand::new(seed);

        let mut tunnel = Tunnel::new(Torus::new(seed));

        let mut camera = Camera::new();
        let mut ship = Ship::new(&screen, seed);

        let mut shots = ShotPool::new(64, &screen);

        let mut stage_manager = StageManager::new(seed);

        let mut manager = GameManager::new(&screen)?;

        let mut events = Events::new(EventSettings::new().swap_buffers(true));

        manager.start(&mut StartParams {
            seed: rand.gen_usize(usize::max_value()) as u64,
            pref_manager: &mut pref_manager,
            stage_manager: &mut stage_manager,
            camera: &mut camera,
            ship: &mut ship,
            tunnel: &mut tunnel,
            shots: &mut shots,
        });

        let start_time = Instant::now();
        let mut prev_millis = 0;

        while let Some(e) = events.next(
            screen
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
                let action = manager.mov(&mut MoveParams {
                    pref_manager: &mut pref_manager,
                    pad: &pad,
                    stage_manager: &mut stage_manager,
                    camera: &mut camera,
                    ship: &mut ship,
                    tunnel: &mut tunnel,
                    shots: &mut shots,
                });
                match action {
                    MoveAction::StartTitle(from_game_over) => {
                        manager.start_title(
                            &mut StartParams {
                                seed: rand.gen_usize(usize::max_value()) as u64,
                                pref_manager: &mut pref_manager,
                                stage_manager: &mut stage_manager,
                                camera: &mut camera,
                                ship: &mut ship,
                                tunnel: &mut tunnel,
                                shots: &mut shots,
                            },
                            from_game_over,
                        );
                    }
                    MoveAction::StartInGame => {
                        manager.start_in_game(&mut StartParams {
                            seed: rand.gen_usize(usize::max_value()) as u64,
                            pref_manager: &mut pref_manager,
                            stage_manager: &mut stage_manager,
                            camera: &mut camera,
                            ship: &mut ship,
                            tunnel: &mut tunnel,
                            shots: &mut shots,
                        });
                    }
                    MoveAction::BreakLoop => self.done = true,
                    MoveAction::None => (),
                }
            }

            if let Some(r) = e.resize_args() {
                screen.resized(r[0], r[1]);
            }

            if let Some(r) = e.render_args() {
                manager.draw(
                    &mut DrawParams {
                        pref_manager: &pref_manager,
                        screen: &screen,
                        letter: &letter,
                        stage_manager: &stage_manager,
                        camera: &mut camera,
                        ship: &mut ship,
                        tunnel: &mut tunnel,
                        shots: &mut shots,
                    },
                    &r,
                );
            }

            if let Some(b) = e.button_args() {
                pad.handle_button_event(&b);
            }

            if self.done {
                break;
            }
        }

        manager.quit_last(&pref_manager)
    }
}

fn main() {
    MainLoop::new().main().unwrap();
}
