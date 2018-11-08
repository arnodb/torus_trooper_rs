use crate::util::rand::Rand;

use crate::tt::tunnel::{SliceColor, SliceDrawState, Torus, Tunnel};

const LEVEL_UP_RATIO: f32 = 0.5;

const TUNNEL_COLOR_CHANGE_INTERVAL: u32 = 60;

const TUNNEL_COLOR_PATTERN_POLY: [SliceColor; 7] = [
    SliceColor {
        r: 0.7,
        g: 0.9,
        b: 1.,
    },
    SliceColor {
        r: 0.6,
        g: 1.,
        b: 0.8,
    },
    SliceColor {
        r: 0.9,
        g: 0.7,
        b: 0.6,
    },
    SliceColor {
        r: 0.8,
        g: 0.8,
        b: 0.8,
    },
    SliceColor {
        r: 0.5,
        g: 0.9,
        b: 0.9,
    },
    SliceColor {
        r: 0.7,
        g: 0.9,
        b: 0.6,
    },
    SliceColor {
        r: 0.8,
        g: 0.5,
        b: 0.9,
    },
];

const TUNNEL_COLOR_PATTERN_LINE: [SliceColor; 7] = [
    SliceColor {
        r: 0.6,
        g: 0.7,
        b: 1.,
    },
    SliceColor {
        r: 0.4,
        g: 0.8,
        b: 0.6,
    },
    SliceColor {
        r: 0.7,
        g: 0.5,
        b: 0.6,
    },
    SliceColor {
        r: 0.6,
        g: 0.6,
        b: 0.6,
    },
    SliceColor {
        r: 0.4,
        g: 0.7,
        b: 0.7,
    },
    SliceColor {
        r: 0.6,
        g: 0.7,
        b: 0.5,
    },
    SliceColor {
        r: 0.6,
        g: 0.4,
        b: 1.,
    },
];

pub struct StageManager {
    rand: Rand,
    level: f32,
    grade: u32,
    middle_boss_zone: bool,
    tunnel_color_poly_idx: usize,
    tunnel_color_line_idx: usize,
    tunnel_color_change_cnt: u32,
    dark_line: bool,
    slice_draw_state: SliceDrawState,
}

impl StageManager {
    pub fn new(seed: u64) -> Self {
        let tunnel_color_poly_idx = TUNNEL_COLOR_PATTERN_POLY.len() - 2;
        let tunnel_color_line_idx = TUNNEL_COLOR_PATTERN_LINE.len() - 2;
        StageManager {
            rand: Rand::new(seed),
            level: 1.,
            grade: 0,
            middle_boss_zone: false,
            tunnel_color_poly_idx,
            tunnel_color_line_idx,
            tunnel_color_change_cnt: TUNNEL_COLOR_CHANGE_INTERVAL,
            dark_line: true,
            slice_draw_state: SliceDrawState {
                dark_line_ratio: 1.,
                poly: TUNNEL_COLOR_PATTERN_POLY[tunnel_color_poly_idx],
                line: TUNNEL_COLOR_PATTERN_POLY[tunnel_color_line_idx],
            },
        }
    }

    pub fn start(&mut self, level: f32, grade: u32, seed: u64, tunnel: &mut Tunnel) {
        self.rand.set_seed(seed);
        tunnel.start(Torus::new(seed));
        self.level = level - LEVEL_UP_RATIO;
        self.grade = grade;
        self.middle_boss_zone = false;
        self.dark_line = true;
        self.tunnel_color_poly_idx = TUNNEL_COLOR_PATTERN_POLY.len() + level as usize - 2;
        self.tunnel_color_line_idx = TUNNEL_COLOR_PATTERN_LINE.len() + level as usize - 2;
        self.slice_draw_state = SliceDrawState {
            dark_line_ratio: 1.,
            poly: TUNNEL_COLOR_PATTERN_POLY[self.tunnel_color_poly_idx % TUNNEL_COLOR_PATTERN_POLY.len()],
            line: TUNNEL_COLOR_PATTERN_LINE[self.tunnel_color_line_idx % TUNNEL_COLOR_PATTERN_LINE.len()],
        };
        self.create_next_zone();
    }

    fn create_next_zone(&mut self) {
        self.level += LEVEL_UP_RATIO;
        self.middle_boss_zone = !self.middle_boss_zone;
        if self.dark_line {
            self.tunnel_color_poly_idx += 1;
            self.tunnel_color_line_idx += 1;
        }
        self.dark_line = !self.dark_line;
        self.tunnel_color_change_cnt = TUNNEL_COLOR_CHANGE_INTERVAL;
        /*
        enemies.clear();
        self.close_ship_spec();
        smallShipSpec = null;
        for (int
        i = 0;
        i < 2 + rand.nextInt(2);
        i + +) {
            ShipSpec ss = new ShipSpec;
            ss.createSmall(rand, _level * 1.8f, grade);
            smallShipSpec ~ = ss;
        }
        middleShipSpec = null;
        for (int
        i = 0;
        i < 2 + rand.nextInt(2);
        i + +) {
            ShipSpec ss = new ShipSpec;
            ss.createMiddle(rand, _level * 1.9f);
            middleShipSpec ~ = ss;
        }
        nextSmallAppDist = nextMiddleAppDist = 0;
        setNextSmallAppDist();
        setNextMiddleAppDist();
        bossShipSpec = null;
        if (_middleBossZone && _level > 5 && rand.nextInt(3) != 0) {
            bossNum = 1 + rand.nextInt(cast(int) sqrt(_level / 5) + 1);
            if (bossNum > 4)
            bossNum = 4;
        } else {
            bossNum = 1;
        }
        for (int
        i = 0;
        i < bossNum;
        i + +) {
            ShipSpec ss = new ShipSpec;
            float lv = _level * 2.0f / bossNum;
            if (_middleBossZone)
            lv *= 1.33f;
            ss.createBoss(rand, lv,
                          0.8 + grade * 0.04 + rand.nextFloat(0.03), _middleBossZone);
            bossShipSpec ~ = ss;
        }
        bossAppRank = BOSS_APP_RANK[grade] - bossNum + zoneEndRank;
        zoneEndRank += BOSS_APP_RANK[grade];
        ship.setBossApp(bossAppRank, bossNum, zoneEndRank);
        bossSpecIdx = 0;
        nextBossAppDist = 9999999;
        bossModeEndCnt = -1;
        */
    }

    pub fn mov(&mut self) {
        // TODO
        if self.tunnel_color_change_cnt > 0 {
            self.tunnel_color_change_cnt -= 1;
            if self.dark_line {
                self.slice_draw_state.dark_line_ratio += 1.0 / TUNNEL_COLOR_CHANGE_INTERVAL as f32;
            } else {
                self.slice_draw_state.dark_line_ratio -= 1.0 / TUNNEL_COLOR_CHANGE_INTERVAL as f32;
                let c_ratio =
                    self.tunnel_color_change_cnt as f32 / TUNNEL_COLOR_CHANGE_INTERVAL as f32;
                let cp_idx_prev =
                    (self.tunnel_color_poly_idx - 1) % TUNNEL_COLOR_PATTERN_POLY.len();
                let cp_idx_now = self.tunnel_color_poly_idx % TUNNEL_COLOR_PATTERN_POLY.len();
                self.slice_draw_state.poly = SliceColor {
                    r: TUNNEL_COLOR_PATTERN_POLY[cp_idx_prev].r * c_ratio
                        + TUNNEL_COLOR_PATTERN_POLY[cp_idx_now].r * (1. - c_ratio),
                    g: TUNNEL_COLOR_PATTERN_POLY[cp_idx_prev].g * c_ratio
                        + TUNNEL_COLOR_PATTERN_POLY[cp_idx_now].g * (1. - c_ratio),
                    b: TUNNEL_COLOR_PATTERN_POLY[cp_idx_prev].b * c_ratio
                        + TUNNEL_COLOR_PATTERN_POLY[cp_idx_now].b * (1. - c_ratio),
                };
                let cl_idx_prev =
                    (self.tunnel_color_line_idx - 1) % TUNNEL_COLOR_PATTERN_LINE.len();
                let cl_idx_now = self.tunnel_color_line_idx % TUNNEL_COLOR_PATTERN_LINE.len();
                self.slice_draw_state.line = SliceColor {
                    r: TUNNEL_COLOR_PATTERN_LINE[cl_idx_prev].r * c_ratio
                        + TUNNEL_COLOR_PATTERN_LINE[cl_idx_now].r * (1. - c_ratio),
                    g: TUNNEL_COLOR_PATTERN_LINE[cl_idx_prev].g * c_ratio
                        + TUNNEL_COLOR_PATTERN_LINE[cl_idx_now].g * (1. - c_ratio),
                    b: TUNNEL_COLOR_PATTERN_LINE[cl_idx_prev].b * c_ratio
                        + TUNNEL_COLOR_PATTERN_LINE[cl_idx_now].b * (1. - c_ratio),
                };
            }
        }
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    pub fn middle_boss_zone(&self) -> bool {
        self.middle_boss_zone
    }

    pub fn slice_draw_state(&self) -> &SliceDrawState {
        &self.slice_draw_state
    }
}
