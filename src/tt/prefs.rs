use preferences::{AppInfo, Preferences};

use crate::tt::errors::GameError;
use crate::tt::ship;

pub struct PrefManager {
    prefs: GamePreferences,
}

impl PrefManager {
    pub fn new() -> Self {
        let prefs = match GamePreferences::load(&APP_INFO, "prefs") {
            Ok(mut prefs) => {
                prefs.clean();
                prefs
            }
            Err(err) => {
                println!("Preferences error: {:?}", err);
                GamePreferences::new()
            }
        };
        PrefManager { prefs }
    }

    pub fn save(&self) -> Result<(), GameError> {
        Ok(self.prefs.save(&APP_INFO, "prefs")?)
    }

    pub fn max_level(&self, gd: u32) -> u32 {
        self.prefs.grade_data[gd as usize].reached_level
    }

    pub fn grade_data(&self, gd: u32) -> &GradeData {
        &self.prefs.grade_data[gd as usize]
    }

    pub fn selected_grade(&self) -> u32 {
        self.prefs.selected_grade
    }

    pub fn selected_level(&self) -> u32 {
        self.prefs.selected_level
    }

    pub fn record_start_game(&mut self, gd: u32, lv: u32) {
        self.prefs.selected_grade = gd;
        self.prefs.selected_level = lv;
    }

    pub fn record_result(&mut self, lv: u32, sc: u32) {
        let gd = &mut self.prefs.grade_data[self.selected_grade() as usize];
        if sc > gd.hi_score {
            gd.hi_score = sc;
            gd.start_level = self.prefs.selected_level;
            gd.end_level = lv;
        }
        if lv > gd.reached_level {
            gd.reached_level = lv;
        }
        self.prefs.selected_level = lv;
    }
}

const APP_INFO: AppInfo = AppInfo {
    name: "tt",
    author: "Torus Trooper",
};

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct GamePreferences {
    selected_grade: u32,
    selected_level: u32,
    grade_data: [GradeData; ship::GRADE_NUM],
}

impl GamePreferences {
    pub fn new() -> Self {
        GamePreferences {
            selected_grade: 0,
            selected_level: 1,
            grade_data: [GradeData::new(); ship::GRADE_NUM],
        }
    }
    pub fn clean(&mut self) {
        if self.selected_grade > ship::GRADE_NUM as u32 {
            self.selected_grade = 0;
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct GradeData {
    pub reached_level: u32,
    pub hi_score: u32,
    pub start_level: u32,
    pub end_level: u32,
}

impl GradeData {
    fn new() -> Self {
        GradeData {
            reached_level: 1,
            hi_score: 0,
            start_level: 1,
            end_level: 1,
        }
    }
}
