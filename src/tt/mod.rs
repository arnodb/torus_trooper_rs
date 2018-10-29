pub mod bullet;
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

// TODO merge all these

pub struct StartParams<'a> {
    pub seed: u64,
    pub pref_manager: &'a mut prefs::PrefManager,

    pub stage_manager: &'a mut manager::stage::StageManager,

    pub camera: &'a mut camera::Camera,
    pub ship: &'a mut ship::Ship,
    pub tunnel: &'a mut tunnel::Tunnel,
}

pub struct MoveParams<'a> {
    pub pref_manager: &'a mut prefs::PrefManager,

    pub pad: &'a pad::Pad,

    pub stage_manager: &'a mut manager::stage::StageManager,

    pub camera: &'a mut camera::Camera,
    pub ship: &'a mut ship::Ship,
    pub tunnel: &'a mut tunnel::Tunnel,
}

pub struct DrawParams<'a> {
    pub pref_manager: &'a prefs::PrefManager,

    pub screen: &'a screen::Screen,
    pub letter: &'a letter::Letter,

    pub stage_manager: &'a manager::stage::StageManager,

    pub camera: &'a camera::Camera,
    pub ship: &'a mut ship::Ship,
    pub tunnel: &'a mut tunnel::Tunnel,
}
