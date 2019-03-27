use crate::util::rand::Rand;

use crate::tt::actor::enemy::ship_spec::ShipSpec;
use crate::tt::actor::enemy::{Enemy, EnemySetOption};
use crate::tt::barrage::BarrageManager;
use crate::tt::screen::Screen;
use crate::tt::ship;
use crate::tt::tunnel::{SliceColor, SliceDrawState, Torus, Tunnel};
use crate::tt::MoreParams;

const BOSS_APP_RANK: [u32; ship::GRADE_NUM] = [100, 160, 250];

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
    next_small_app_dist: f32,
    next_medium_app_dist: f32,
    next_boss_app_dist: f32,
    boss_num: u32,
    zone_end_rank: u32,
    level: f32,
    grade: u32,
    boss_mode_end_cnt: i32,
    medium_boss_zone: bool,
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
            next_small_app_dist: 0.,
            next_medium_app_dist: 0.,
            next_boss_app_dist: 0.,
            boss_num: 0,
            zone_end_rank: 0,
            level: 1.,
            grade: 0,
            boss_mode_end_cnt: 0,
            medium_boss_zone: false,
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

    pub fn start(
        &mut self,
        level: f32,
        grade: u32,
        seed: u64,
        screen: &Screen,
        tunnel: &mut Tunnel,
        barrage_manager: &mut BarrageManager,
        more_params: &mut MoreParams,
    ) {
        self.rand.set_seed(seed);
        tunnel.start(Torus::new(seed));
        self.level = level - LEVEL_UP_RATIO;
        self.grade = grade;
        self.zone_end_rank = 0;
        self.medium_boss_zone = false;
        self.dark_line = true;
        self.tunnel_color_poly_idx = TUNNEL_COLOR_PATTERN_POLY.len() + level as usize - 2;
        self.tunnel_color_line_idx = TUNNEL_COLOR_PATTERN_LINE.len() + level as usize - 2;
        self.slice_draw_state = SliceDrawState {
            dark_line_ratio: 1.,
            poly: TUNNEL_COLOR_PATTERN_POLY
                [self.tunnel_color_poly_idx % TUNNEL_COLOR_PATTERN_POLY.len()],
            line: TUNNEL_COLOR_PATTERN_LINE
                [self.tunnel_color_line_idx % TUNNEL_COLOR_PATTERN_LINE.len()],
        };
        self.create_next_zone(screen, barrage_manager, more_params);
    }

    fn create_next_zone(
        &mut self,
        screen: &Screen,
        barrage_manager: &mut BarrageManager,
        more_params: &mut MoreParams,
    ) {
        self.level += LEVEL_UP_RATIO;
        self.medium_boss_zone = !self.medium_boss_zone;
        if self.dark_line {
            self.tunnel_color_poly_idx += 1;
            self.tunnel_color_line_idx += 1;
        }
        self.dark_line = !self.dark_line;
        self.tunnel_color_change_cnt = TUNNEL_COLOR_CHANGE_INTERVAL;
        more_params.enemies.clear(more_params.bullets);
        self.next_small_app_dist = 0.;
        self.next_medium_app_dist = 0.;
        self.set_next_small_app_dist();
        self.set_next_medium_app_dist();
        if self.medium_boss_zone && self.level > 5. && self.rand.gen_usize(3) != 0 {
            self.boss_num = 1 + self.rand.gen_usize(f32::sqrt(self.level / 5.) as usize + 1) as u32;
            if self.boss_num > 4 {
                self.boss_num = 4;
            }
        } else {
            self.boss_num = 1;
        }
        more_params.enemies.renew_ship_specs(
            self.level,
            self.grade,
            self.medium_boss_zone,
            self.boss_num,
            screen,
            barrage_manager,
        );
        let boss_app_rank = BOSS_APP_RANK[self.grade as usize] - self.boss_num + self.zone_end_rank;
        self.zone_end_rank += BOSS_APP_RANK[self.grade as usize];
        more_params
            .ship
            .set_boss_app(boss_app_rank, self.boss_num, self.zone_end_rank);
        self.next_boss_app_dist = 9999999.;
        self.boss_mode_end_cnt = -1;
    }

    pub fn mov(
        &mut self,
        screen: &Screen,
        tunnel: &Tunnel,
        barrage_manager: &mut BarrageManager,
        more_params: &mut MoreParams,
    ) {
        if more_params.ship.in_boss_mode() {
            if self.next_boss_app_dist > 99999. {
                self.next_boss_app_dist = (self.rand.gen_usize(50) + 100) as f32;
                self.next_small_app_dist = 9999999.;
                self.next_medium_app_dist = 9999999.;
            }
            self.next_boss_app_dist -= more_params.ship.speed();
            if self.boss_num > 0 && self.next_boss_app_dist <= 0. {
                more_params.enemies.get_boss_instance_and(|enemy, spec| {
                    self.add_enemy(ship::IN_SIGHT_DEPTH_DEFAULT * 4., enemy, spec, tunnel);
                });
                self.boss_num -= 1;
                self.next_boss_app_dist = (self.rand.gen_usize(30) + 60) as f32;
            }
            if self.boss_num <= 0 && more_params.enemies.get_num() <= 0 {
                more_params.ship.goto_next_zone_forced();
            }
            return;
        } else {
            if self.next_boss_app_dist < 99999. {
                // Player's ship destroyed or overtook all bosses.
                self.boss_mode_end_cnt = 60;
                self.next_small_app_dist = 9999999.;
                self.next_medium_app_dist = 9999999.;
                self.next_boss_app_dist = 9999999.;
            }
            if self.boss_mode_end_cnt >= 0 {
                self.boss_mode_end_cnt -= 1;
                more_params.bullets.clear_visible();
                if self.boss_mode_end_cnt < 0 {
                    self.create_next_zone(screen, barrage_manager, more_params);
                    more_params.ship.start_next_zone();
                }
            }
        }
        self.next_small_app_dist -= more_params.ship.speed();
        if self.next_small_app_dist <= 0. {
            let y = ship::IN_SIGHT_DEPTH_DEFAULT * (4. + self.rand.gen_f32(0.5));
            more_params.enemies.get_small_instance_and(|enemy, spec| {
                self.add_enemy(y, enemy, spec, tunnel);
            });
            self.set_next_small_app_dist();
        }
        self.next_medium_app_dist -= more_params.ship.speed();
        if self.next_medium_app_dist <= 0. {
            let y = ship::IN_SIGHT_DEPTH_DEFAULT * (4. + self.rand.gen_f32(0.5));
            more_params.enemies.get_medium_instance_and(|enemy, spec| {
                self.add_enemy(y, enemy, spec, tunnel);
            });
            self.set_next_medium_app_dist();
        }
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

    pub fn set_next_small_app_dist(&mut self) {
        self.next_small_app_dist += (self.rand.gen_usize(16) + 6) as f32;
    }

    pub fn set_next_medium_app_dist(&mut self) {
        self.next_medium_app_dist += (self.rand.gen_usize(200) + 33) as f32;
    }

    fn add_enemy(&mut self, y: f32, enemy: &mut Enemy, spec: &ShipSpec, tunnel: &Tunnel) {
        let sl = tunnel.get_slice(y);
        let mut x = if sl.is_nearly_round() {
            self.rand.gen_f32(std::f32::consts::PI)
        } else {
            let ld = sl.get_left_edge_deg();
            let rd = sl.get_right_edge_deg();
            let mut wd = rd - ld;
            if wd < 0. {
                wd += std::f32::consts::PI * 2.;
            }
            ld + self.rand.gen_f32(wd)
        };
        if x < 0. {
            x += std::f32::consts::PI * 2.;
        } else if x >= std::f32::consts::PI * 2. {
            x -= std::f32::consts::PI * 2.;
        }
        enemy.set(
            x,
            y,
            EnemySetOption::New {
                spec,
                rand: &mut self.rand,
            },
        );
    }

    pub fn level(&self) -> f32 {
        self.level
    }

    pub fn medium_boss_zone(&self) -> bool {
        self.medium_boss_zone
    }

    pub fn slice_draw_state(&self) -> &SliceDrawState {
        &self.slice_draw_state
    }
}
