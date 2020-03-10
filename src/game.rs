use ggez::{
    event::EventHandler,
    graphics::{self, Color, DrawParam, Image},
    mint::{Point2, Vector2},
    timer::time_since_start,
    Context, GameResult,
};
use winit::MouseButton;

use std::{fmt, path::Path, time::Duration};

use crate::grid::{BoardState, Grid};

pub const UI_SCALE: f32 = 4.0;

#[derive(PartialEq)]
pub enum PlayState {
    Unstarted,
    Playing(Duration),
    Won(Duration),
    Lost(Duration),
}

pub struct GameState {
    total_mines: i32,
    total_flags: i32,
    turns: i32,
    play_state: PlayState,
    grid: Grid,
    spritesheet: Vec<Image>,
}

impl GameState {
    pub fn new(total_mines: i32, grid: Grid, spritesheet: Vec<Image>) -> Self {
        GameState {
            total_mines,
            total_flags: 0,
            turns: 0,
            play_state: PlayState::Unstarted,
            grid,
            spritesheet,
        }
    }
}

impl EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let grid_x = (x / (8. * UI_SCALE)) as i32;
        let grid_y = (y / (8. * UI_SCALE) - 3.) as i32;
        match button {
            MouseButton::Left => {
                if y >= (24. * UI_SCALE) {
                    if self.play_state == PlayState::Unstarted {
                        self.play_state = PlayState::Playing(time_since_start(ctx));
                        if let Some(cell) = self.grid.get(grid_x, grid_y) {
                            if cell.has_mine {}
                        }
                    }
                    match self.grid.uncover(grid_x, grid_y) {
                        BoardState::InProgress => {}
                        BoardState::Cleared => {
                            self.play_state = PlayState::Won(time_since_start(ctx))
                        }
                        BoardState::Detonated => {
                            self.play_state = PlayState::Lost(time_since_start(ctx))
                        }
                    };
                    self.turns += 1;
                }
            }
            MouseButton::Right => {
                self.total_flags += self.grid.toggle_flag(grid_x, grid_y);
            }
            _ => {}
        }
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(60. / 255., 50. / 255., 83. / 255., 1.));

        // Set UI scale
        let transform = DrawParam::new()
            .scale(Vector2 {
                x: UI_SCALE,
                y: UI_SCALE,
            })
            .to_matrix();
        graphics::set_transform(ctx, transform);
        let _ = graphics::apply_transformations(ctx);

        // Draw UI
        let mut cursor_x = 1;
        {
            // Draw Turn Counter
            let sprite_params = DrawParam::new().dest(Point2 {
                x: cursor_x as f32 * 8.,
                y: 1 as f32 * 8.,
            });
            graphics::draw(ctx, &self.spritesheet[15], sprite_params)?;
            cursor_x += 1;
            let seconds_since_start = match self.play_state {
                PlayState::Won(end_time) => end_time.as_secs(),
                PlayState::Lost(end_time) => end_time.as_secs(),
                PlayState::Playing(start_time) => (time_since_start(ctx) - start_time).as_secs(),
                PlayState::Unstarted => 0,
            };
            for sprite in number_to_sprites(seconds_since_start as i32) {
                let sprite_params = DrawParam::new().dest(Point2 {
                    x: cursor_x as f32 * 8.,
                    y: 1 as f32 * 8.,
                });
                graphics::draw(ctx, &self.spritesheet[sprite as usize], sprite_params)?;
                cursor_x += 1;
            }

            cursor_x += 1;
            // Draw Flag Counter
            let sprite_params = DrawParam::new().dest(Point2 {
                x: cursor_x as f32 * 8.,
                y: 1 as f32 * 8.,
            });
            graphics::draw(ctx, &self.spritesheet[11], sprite_params)?;
            cursor_x += 1;
            for sprite in number_to_sprites(self.total_flags) {
                let sprite_params = DrawParam::new().dest(Point2 {
                    x: cursor_x as f32 * 8.,
                    y: 1 as f32 * 8.,
                });
                graphics::draw(ctx, &self.spritesheet[sprite as usize], sprite_params)?;
                cursor_x += 1;
            }

            cursor_x += 1;
            // Draw Mine Counter
            let sprite_params = DrawParam::new().dest(Point2 {
                x: cursor_x as f32 * 8.,
                y: 1 as f32 * 8.,
            });
            graphics::draw(ctx, &self.spritesheet[10], sprite_params)?;
            cursor_x += 1;
            for sprite in number_to_sprites(self.total_mines) {
                let sprite_params = DrawParam::new().dest(Point2 {
                    x: cursor_x as f32 * 8.,
                    y: 1 as f32 * 8.,
                });
                graphics::draw(ctx, &self.spritesheet[sprite as usize], sprite_params)?;
                cursor_x += 1;
            }
        }

        // Draw minefield
        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let sprite_params = DrawParam::new().dest(Point2 {
                    x: x as f32 * 8.,
                    y: (y as f32 * 8. + 24.),
                });
                graphics::draw(
                    ctx,
                    &self.spritesheet[self.grid.get(x, y).unwrap().sprite_index()],
                    sprite_params,
                )?;
            }
        }

        graphics::present(ctx)
    }
}

fn number_to_sprites(x: i32) -> Vec<u8> {
    x.to_string()
        .chars()
        .map(|digit| digit.to_string().parse::<u8>().unwrap())
        .collect::<Vec<u8>>()
}
