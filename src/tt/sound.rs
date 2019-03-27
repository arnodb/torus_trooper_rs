use failure::Backtrace;
use sdl2::mixer::{Channel, Chunk, Music, AUDIO_S16};
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;

use crate::tt::errors::SoundError;
use crate::util::rand::Rand;
use sdl2::Sdl;
use std::path::Path;

pub struct SoundManager<'a> {
    no_sound: bool,
    se_disabled: bool,
    bgm: Vec<Music<'a>>,
    se: BTreeMap<String, (Chunk, i32)>,
    prev_bgm_idx: usize,
    next_idx_mv: isize,
    rand: Rand,
    sdl: Option<Sdl>,
}

const MUSIC_DIR_NAME: &str = "sounds/musics";
const CHUNK_DIR_NAME: &str = "sounds/chunks";

const FADE_OUT_SPEED: i32 = 1280;

impl<'a> SoundManager<'a> {
    pub fn new(no_sound: bool) -> Result<Self, SoundError> {
        Ok(SoundManager {
            no_sound,
            se_disabled: false,
            bgm: Vec::new(),
            se: BTreeMap::new(),
            prev_bgm_idx: 0,
            next_idx_mv: 0,
            rand: Rand::new(Rand::rand_seed()),
            sdl: None,
        })
    }

    pub fn init(&mut self, no_sound: bool) -> Result<(), SoundError> {
        if no_sound {
            return Ok(());
        }
        let sdl = sdl2::init().map_err(|str| SoundError::Sdl(str, Backtrace::new()))?;
        let frequency = 44_100;
        let format = AUDIO_S16;
        let channels = 1;
        let chunk_size = 4_096;
        sdl2::mixer::open_audio(frequency, format, channels, chunk_size)
            .map_err(|str| SoundError::Sdl(str, Backtrace::new()))?;

        self.bgm = SoundManager::load_musics()?;
        self.se = SoundManager::load_chunks()?;
        self.prev_bgm_idx = self.bgm.len();

        self.sdl = Some(sdl);

        Ok(())
    }

    fn load_musics<'m>() -> Result<Vec<Music<'m>>, SoundError> {
        let mut musics = Vec::new();
        let files = fs::read_dir(MUSIC_DIR_NAME)?;
        for file_name in files {
            let file_name = file_name?;
            if file_name.file_type()?.is_file() {
                match file_name.path().extension().and_then(OsStr::to_str) {
                    Some("ogg") | Some("wav") => {
                        let music = Music::from_file(file_name.path())
                            .map_err(|str| SoundError::Sdl(str, Backtrace::new()))?;
                        musics.push(music);
                    }
                    _ => {}
                }
            }
        }
        Ok(musics)
    }

    fn load_chunks() -> Result<BTreeMap<String, (Chunk, i32)>, SoundError> {
        let mut chunks = BTreeMap::new();
        let chunk_path = Path::new(CHUNK_DIR_NAME);
        for (file_name, ch) in vec![
            ("shot.wav", 0),
            ("charge.wav", 1),
            ("charge_shot.wav", 1),
            ("hit.wav", 2),
            ("small_dest.wav", 3),
            ("middle_dest.wav", 4),
            ("boss_dest.wav", 4),
            ("myship_dest.wav", 5),
            ("extend.wav", 6),
            ("timeup_beep.wav", 7),
        ] {
            let chunk = Chunk::from_file(chunk_path.join(file_name))
                .map_err(|str| SoundError::Sdl(str, Backtrace::new()))?;
            chunks.insert(file_name.to_string(), (chunk, ch));
        }
        Ok(chunks)
    }

    pub fn set_rand_seed(&mut self, seed: u64) {
        self.rand.set_seed(seed)
    }

    pub fn play_bgm(&mut self) {
        if self.no_sound {
            return;
        }
        let mut bgm_idx = self.rand.gen_usize(self.bgm.len());
        self.next_idx_mv = self.rand.gen_usize(2) as isize * 2 - 1;
        if bgm_idx == self.prev_bgm_idx {
            bgm_idx += 1;
            if bgm_idx >= self.bgm.len() {
                bgm_idx = 0;
            }
        }
        self.prev_bgm_idx = bgm_idx;
        unwrap_sdl_error(self.bgm[bgm_idx].play(-1));
    }

    pub fn next_bgm(&mut self) {
        if self.no_sound {
            return;
        }
        let mut bgm_idx = self.prev_bgm_idx as isize + self.next_idx_mv;
        if bgm_idx < 0 {
            bgm_idx = self.bgm.len() as isize - 1;
        } else if bgm_idx >= self.bgm.len() as isize {
            bgm_idx = 0;
        }
        self.prev_bgm_idx = bgm_idx as usize;
        unwrap_sdl_error(self.bgm[bgm_idx as usize].play(-1));
    }

    pub fn fade_bgm(&self) {
        if self.no_sound {
            return;
        }
        unwrap_sdl_error(Music::fade_out(FADE_OUT_SPEED));
    }

    pub fn halt_bgm(&self) {
        if self.no_sound {
            return;
        }
        Music::halt()
    }

    pub fn enable_se(&mut self) {
        self.se_disabled = false;
    }

    pub fn disable_se(&mut self) {
        self.se_disabled = true;
    }

    // TODO use constants to call play_se
    pub fn play_se(&self, name: &str) {
        if self.no_sound || self.se_disabled {
            return;
        }
        let se = self.se.get(name);
        if let Some((chunk, ch)) = se {
            unwrap_sdl_error(Channel(*ch).play(chunk, 0));
        } else {
            eprintln!("Sound error: did not load sound effect {}", name);
        }
    }
}

fn unwrap_sdl_error<T>(res: Result<T, String>) {
    res.map(|_| ())
        .unwrap_or_else(|str| eprintln!("SDL error: {}", str));
}

impl<'a> Drop for SoundManager<'a> {
    fn drop(&mut self) {
        if self.no_sound {
            return;
        }
        if Music::is_playing() {
            Music::halt();
        }
        for (_, ch) in self.se.values() {
            let channel = Channel(*ch);
            if channel.is_playing() {
                channel.halt();
            }
        }
        sdl2::mixer::close_audio();
    }
}
