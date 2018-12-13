use piston::input::RenderArgs;

use crate::gl;

use crate::tt::errors::GameError;
use crate::tt::manager::title::TitleManager;
use crate::tt::manager::{Manager, MoveAction};
use crate::tt::screen::Screen;
use crate::tt::ActionParams;

use super::State;

pub struct TitleState {
    manager: TitleManager,
    game_over_cnt: u32,
}

impl TitleState {
    pub fn new(screen: &Screen) -> Result<Self, GameError> {
        Ok(TitleState {
            manager: TitleManager::new(screen)?,
            game_over_cnt: 0,
        })
    }

    fn clear_all(&mut self, params: &mut ActionParams) {
        params.shots.clear();
        params.bullets.clear();
        params.enemies.clear_shallow();
        params.particles.clear();
        params.float_letters.clear();
        /*TODO
        passedEnemies.clear();
        */
    }
}

impl State for TitleState {
    fn start(&mut self, seed: u64, params: &mut ActionParams) {
        // TODO SoundManager.haltBgm();
        // TODO SoundManager.disableSe();
        self.manager.start(seed, params);
        self.clear_all(params);
        // TODO if (replayData)
        // TODO startReplay();
    }

    fn mov(&mut self, params: &mut ActionParams) -> MoveAction {
        if params.ship.is_game_over() {
            self.game_over_cnt += 1;
            if self.game_over_cnt > 120 {
                // TODO clearAll();
                // TODO startReplay();
            }
        }
        /* TODO
        if (replayData) {
            ship.move();
            stageManager.move();
            enemies.move();
            shots.move();
            bullets.move();
            particles.move();
            floatLetters.move();
            passedEnemies.move();
            inGameState.decrementTime();
            titleManager.move(true);
        } else {
            titleManager.move(false);
        }*/
        let action = self.manager.mov(false, params);
        action
    }

    fn draw(&self, params: &mut ActionParams, render_args: &RenderArgs) {
        /*TODO
            if (replayData) {
          float rcr = titleManager.replayChangeRatio * 2.4f;
          if (rcr > 1)
            rcr = 1;
          glViewport(0, 0,
                     cast(int) (Screen.width / 4 * (3 + rcr)),
                     Screen.height);
          glEnable(GL_CULL_FACE);
          tunnel.draw();
          tunnel.drawBackward();
          glDisable(GL_CULL_FACE);
          particles.draw();
          enemies.draw();
          passedEnemies.draw();
          ship.draw();
          glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA);
          floatLetters.draw();
          glBlendFunc(GL_SRC_ALPHA, GL_ONE);
          glDisable(GL_BLEND);
          bullets.draw();
          glEnable(GL_BLEND);
          shots.draw();
        }
        */
        unsafe {
            let screen = &params.screen;
            let p_size = screen.physical_size();
            gl::Viewport(0, 0, p_size.0 as i32, p_size.1 as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            let ratio_threshold = 480. / 640.;
            let screen_ratio = p_size.1 as f32 / p_size.0 as f32;
            if screen_ratio >= ratio_threshold {
                gl::Frustum(
                    -screen.near_plane() as f64,
                    screen.near_plane() as f64,
                    (-screen.near_plane() * screen_ratio) as f64,
                    (screen.near_plane() * screen_ratio) as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
            } else {
                // This allows to see at least what can be seen horizontally and vertically
                // with the default ratio -- arnodb
                gl::Frustum(
                    (-screen.near_plane() * ratio_threshold / screen_ratio) as f64,
                    (screen.near_plane() * ratio_threshold / screen_ratio) as f64,
                    (-screen.near_plane() * ratio_threshold) as f64,
                    (screen.near_plane() * ratio_threshold) as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
            }
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
        self.manager.draw(params, render_args)
    }

    fn draw_front(&self, params: &ActionParams, render_args: &RenderArgs) {
        self.manager.draw_front(params, render_args);
        /*TODO
        if (!ship.drawFrontMode || titleManager.replayChangeRatio < 1)
        return;
        inGameState.drawFront();
        */
    }
}
