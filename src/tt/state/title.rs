use piston::input::RenderArgs;

use crate::gl;

use crate::tt::errors::GameError;
use crate::tt::manager::title::TitleManager;
use crate::tt::manager::{Manager, MoveAction};
use crate::tt::screen::Screen;
use crate::tt::{DrawParams, MoveParams, StartParams};

use super::State;

pub struct TitleState {
    manager: TitleManager,
}

impl TitleState {
    pub fn new(screen: &Screen) -> Result<Self, GameError> {
        Ok(TitleState {
            manager: TitleManager::new(screen)?,
        })
    }
}

impl State for TitleState {
    fn start(&mut self, params: &mut StartParams) {
        // TODO SoundManager.haltBgm();
        // TODO SoundManager.disableSe();
        self.manager.start(params);
        // TODO clearAll();
        // TODO if (replayData)
        // TODO startReplay();
    }

    fn mov(&mut self, params: &mut MoveParams) -> MoveAction {
        // TODO
        let action = self.manager.mov(false, params);
        // TODO
        action
    }

    fn draw(&self, params: &mut DrawParams, render_args: &RenderArgs) {
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
        let screen = params.screen;
        unsafe {
            gl::Viewport(0, 0, screen.width() as i32, screen.height() as i32);
            gl::MatrixMode(gl::GL_PROJECTION);
            gl::LoadIdentity();
            let screen_ratio = screen.height() as f32 / screen.width() as f32;
            if screen_ratio >= 480. / 640. {
                gl::Frustum(
                    -screen.near_plane() as f64,
                    screen.near_plane() as f64,
                    (-screen.near_plane() * screen_ratio) as f64,
                    (screen.near_plane() * screen_ratio) as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
            } else {
                gl::Frustum(
                    (-screen.near_plane() / screen_ratio) as f64,
                    (screen.near_plane() / screen_ratio) as f64,
                    -screen.near_plane() as f64,
                    screen.near_plane() as f64,
                    0.1,
                    screen.far_plane() as f64,
                );
            }
            gl::MatrixMode(gl::GL_MODELVIEW);
        }
        self.manager.draw(params, render_args)
    }

    fn draw_front(&self, params: &DrawParams, render_args: &RenderArgs) {
        self.manager.draw_front(params, render_args);
        /*TODO
        if (!ship.drawFrontMode || titleManager.replayChangeRatio < 1)
        return;
        inGameState.drawFront();
        */
    }
}
