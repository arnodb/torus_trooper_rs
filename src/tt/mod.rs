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
pub mod state;
pub mod tunnel;

pub struct ActionParams<'a> {
    pub pref_manager: &'a mut prefs::PrefManager,

    pub screen: &'a mut screen::Screen,
    pub letter: &'a letter::Letter,

    pub pad: &'a mut pad::Pad,

    pub stage_manager: &'a mut manager::stage::StageManager,

    pub camera: &'a mut camera::Camera,
    pub ship: &'a mut ship::Ship,
    pub tunnel: &'a mut tunnel::Tunnel,

    pub barrage_manager: &'a mut barrage::BarrageManager,
    pub shots: &'a mut actor::shot::ShotPool,
    pub bullets: &'a mut actor::bullet::BulletPool,
    pub enemies: &'a mut actor::enemy::EnemyPool,
}
