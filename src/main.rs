mod game;
mod grid;

use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{FilterMode, Image},
    Context, ContextBuilder, GameError,
};

use std::{fmt, path::Path, time::Duration};

use crate::{
    game::{GameState, PlayState, UI_SCALE},
    grid::Grid,
};

fn main() -> Result<(), GameError> {
    let (ref mut ctx, ref mut event_loop) = ContextBuilder::new("minesweeper", "")
        .window_setup(WindowSetup::default().title("minesweeper"))
        .window_mode(WindowMode::default().dimensions(256. * UI_SCALE, (256. + 24.0) * UI_SCALE))
        .add_resource_path("assets")
        .build()
        .unwrap();
    let spritesheet = load_spritesheet(ctx, "/minesweeper.png", 8, 8, 4)?;

    let width = 32;
    let height = 32;
    let mut grid = Grid::new(width, height);
    let mut mine_count = 0;
    for x in 0..width {
        for y in 0..height {
            if rand::random::<f32>() > 0.85 {
                // TODO: Mines can spawn on the same position and the count
                // would become incorrect
                grid.place_mine(x, y);
                mine_count += 1;
            }
        }
    }

    let state = &mut GameState::new(mine_count, grid, spritesheet);
    event::run(ctx, event_loop, state).unwrap();

    Ok(())
}

fn load_spritesheet(
    ctx: &mut Context,
    path: &str,
    sprite_width: usize,
    sprite_height: usize,
    horizontal_sprite_count: usize,
) -> Result<Vec<Image>, GameError> {
    let sprite_size = sprite_width * sprite_height;
    let sprite_row_size = sprite_width * horizontal_sprite_count;

    // Load and split image into a Vec of sprites
    Ok(Image::new(ctx, Path::new(path))?
        .to_rgba8(ctx)?
        // Split pixel data into sprite rows
        .chunks(sprite_size * horizontal_sprite_count * 4)
        .flat_map(|row| {
            row
                // Split pixel data into sprite row cross-sections
                .chunks(sprite_row_size)
                .enumerate()
                // Merge rows into sprite pixel data
                .fold(
                    vec![vec![]; horizontal_sprite_count],
                    |mut sprites, (i, row)| {
                        sprites[i % horizontal_sprite_count].extend_from_slice(row);
                        sprites
                    },
                )
        })
        // Transform sprite pixel data into images
        .map(|pixels| {
            let mut sprite =
                Image::from_rgba8(ctx, sprite_width as u16, sprite_height as u16, &pixels).unwrap();
            sprite.set_filter(FilterMode::Nearest);
            sprite
        })
        .collect::<Vec<Image>>())
}
