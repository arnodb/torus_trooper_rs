#![cfg_attr(nightly, feature(backtrace))]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate thiserror;

#[macro_use]
mod macros;

pub mod tt;
pub mod util;

#[allow(clippy::all)]
pub mod gl;
pub mod glu;

#[cfg(feature = "game_recorder")]
mod game_recorder;

use piston::event_loop::*;
use piston::input::*;
use std::error::Error;
use std::str::FromStr;
use std::time::Instant;
use structopt::StructOpt;

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
use crate::tt::sound::SoundManager;
use crate::tt::state::shared::SharedState;
use crate::tt::tunnel::{Torus, Tunnel};
use crate::tt::{GeneralParams, MoreParams};
use crate::util::rand::Rand;

struct MainLoop {
    options: Options,
}

impl MainLoop {
    fn new(options: Options) -> Self {
        MainLoop { options }
    }

    fn main(&mut self) -> Result<(), GameError> {
        let sdl = sdl2::init().map_err(GameError::new_sdl2_init)?;
        let sdl_joystick = sdl.joystick().map(Some).unwrap_or_else(|err| {
            eprintln!("{}", err);
            None
        });

        let mut pref_manager = PrefManager::new();

        let mut screen = Screen::new(
            self.options.brightness as f32 / 100.,
            self.options.luminosity as f32 / 100.,
            self.options.fullscreen,
            [self.options.width as u32, self.options.height as u32].into(),
        );
        #[cfg(not(feature = "sdl_backend"))]
        screen.init_opengl()?;
        #[cfg(feature = "sdl_backend")]
        screen.init_opengl_sdl(sdl.video().map_err(GameError::new_sdl2_video_init)?)?;

        let mut pad = GamePad::new(self.options.reverse, sdl_joystick)?;

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

        let mut sound_manager = SoundManager::new(self.options.no_sound);
        sound_manager.init()?;

        let mut manager = GameManager::new(&screen)?;
        let mut shared_state = SharedState::new();

        let mut events = Events::new(EventSettings::new().swap_buffers(true));

        let mut params = GeneralParams {
            pref_manager: &mut pref_manager,
            screen: &mut screen,
            letter: &letter,
            pad: &mut pad,
            shared_state: &mut shared_state,
            stage_manager: &mut stage_manager,
            sound_manager: &mut sound_manager,
            camera: &mut camera,
            tunnel: &mut tunnel,
            barrage_manager: &mut barrage_manager,
            #[cfg(feature = "game_recorder")]
            next_recorder_id: record_next_id!(),
        };
        let mut more_params = MoreParams {
            ship: &mut ship,
            shots: &mut shots,
            bullets: &mut bullets,
            enemies: &mut enemies,
            particles: &mut particles,
            float_letters: &mut float_letters,
        };

        manager.start(&mut params, &mut more_params)?;

        let start_time = Instant::now();
        let mut prev_millis = 0;

        let mut done = false;

        while let Some(e) = events.next(
            params
                .screen
                .window_mut()
                .ok_or_else(|| GameError::new_missing_window())?,
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
                let action = manager.mov(&mut params, &mut more_params);
                match action {
                    MoveAction::StartTitle(from_game_over) => {
                        record_event_end!(from_game_over);
                        record_stop!();
                        manager.start_title(
                            &mut params,
                            &mut more_params,
                            false,
                            from_game_over,
                        )?;
                    }
                    MoveAction::StartInGame => {
                        let new_seed = Rand::rand_seed();
                        manager.start_in_game(new_seed, &mut params, &mut more_params)?;
                    }
                    MoveAction::BreakLoop => done = true,
                    MoveAction::None => (),
                }
            }

            if let Some(r) = e.resize_args() {
                params.screen.resized(r);
            }

            if let Some(r) = e.render_args() {
                Screen::clear();
                manager.draw(&mut params, &mut more_params, &r);
            }

            if let Some(b) = e.button_args() {
                params.pad.handle_button_event(&b);
            }

            if let Some(f) = e.focus_args() {
                params.pad.handle_focus_event(f);
            }

            if done {
                break;
            }
        }

        manager.quit_last(params.pref_manager)
    }
}

fn parse_brightness(s: &str) -> Result<usize, Box<dyn Error>> {
    let val = usize::from_str(s)?;
    if val > 100 {
        Err("brightness must if in the range [0-100]")?;
    }
    Ok(val)
}

fn parse_luminosity(s: &str) -> Result<usize, Box<dyn Error>> {
    let val = usize::from_str(s)?;
    if val > 100 {
        Err("luminosity must if in the range [0-100]")?;
    }
    Ok(val)
}

#[derive(StructOpt, Debug)]
struct Options {
    #[structopt(long, default_value = "100", parse(try_from_str = "parse_brightness"))]
    brightness: usize,
    #[structopt(long, default_value = "0", parse(try_from_str = "parse_luminosity"))]
    luminosity: usize,
    #[structopt(long, default_value = "640")]
    width: usize,
    #[structopt(long, default_value = "480")]
    height: usize,
    #[structopt(long)]
    fullscreen: bool,
    #[structopt(long = "nosound")]
    no_sound: bool,
    #[structopt(long)]
    reverse: bool,
}

fn main() {
    let options = Options::from_args();
    MainLoop::new(options).main().unwrap();
}
