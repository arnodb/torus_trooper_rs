pub mod actor;
pub mod barrage;
pub mod camera;
pub mod errors;
pub mod letter;
pub mod manager;
pub mod pad;
pub mod prefs;
pub mod screen;
pub mod shape;
pub mod ship;
pub mod sound;
pub mod state;
pub mod tunnel;

pub struct GeneralParams<'a, 'shared, 'sound> {
    pub pref_manager: &'a mut prefs::PrefManager,

    pub screen: &'a mut screen::Screen,
    pub letter: &'a letter::Letter,

    pub pad: &'a mut dyn pad::Pad,

    pub shared_state: &'a mut state::shared::SharedState<'shared>,

    pub stage_manager: &'a mut manager::stage::StageManager,

    pub sound_manager: &'a mut sound::SoundManager<'sound>,

    pub camera: &'a mut camera::Camera,
    pub tunnel: &'a mut tunnel::Tunnel,

    pub barrage_manager: &'a mut barrage::BarrageManager,

    #[cfg(feature = "game_recorder")]
    pub next_recorder_id: usize,
}

impl<'a, 'shared, 'sound> GeneralParams<'a, 'shared, 'sound> {
    fn add_score(&mut self, score: u32, game_over: bool) {
        self.shared_state.add_score(
            score,
            game_over,
            self.stage_manager.level(),
            self.sound_manager,
        );
    }
}

// Those params are separated from the general params because they very often interact
// together in all directions. In those cases, the code can still pass general params as
// a single reference.
pub struct MoreParams<'a> {
    pub ship: &'a mut ship::Ship,

    pub shots: &'a mut actor::shot::ShotPool,
    pub bullets: &'a mut actor::bullet::BulletPool,
    pub enemies: &'a mut actor::enemy::EnemyPool,
    pub particles: &'a mut actor::particle::ParticlePool,
    pub float_letters: &'a mut actor::float_letter::FloatLetterPool,
}
