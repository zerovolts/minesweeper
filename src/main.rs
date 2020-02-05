use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{self, Color, DrawParam, FilterMode, Image},
    mint::{Point2, Vector2},
    timer::time_since_start,
    Context, ContextBuilder, GameError, GameResult,
};
use winit::MouseButton;

use std::{fmt, path::Path, time::Duration};

const UI_SCALE: f32 = 4.0;

#[derive(PartialEq)]
enum PlayState {
    Unstarted,
    Playing(Duration),
    Won(Duration),
    Lost(Duration),
}

enum BoardState {
    InProgress,
    Cleared,
    Detonated,
}

struct GameState {
    total_mines: i32,
    total_flags: i32,
    turns: i32,
    play_state: PlayState,
    // percent covered
    grid: Grid,
    spritesheet: Vec<Image>,
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

struct Grid {
    cells: Vec<Cell>,
    width: i32,
    height: i32,
}

const NEIGHBOR_OFFSETS: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
];

impl Grid {
    pub fn new(width: i32, height: i32) -> Self {
        let mut grid = Grid {
            cells: vec![],
            width,
            height,
        };
        for y in 0..height {
            for x in 0..width {
                grid.cells
                    .push(Cell::new(CellState::Covered, false, 0, x, y));
            }
        }
        grid
    }

    pub fn uncover(&mut self, x: i32, y: i32) -> BoardState {
        let index = self.coord_to_index(x, y).unwrap();
        self.cells[index].state = CellState::Exposed;

        if self.cells[index].has_mine == true {
            self.uncover_bombs();
            return BoardState::Detonated;
        }

        // if the cell has no adjacent mines, uncover adjacent cells without adjacent mines
        if self.cells[index].neighboring_mines == 0 && self.cells[index].has_mine == false {
            for (i, j) in NEIGHBOR_OFFSETS.iter() {
                self.coord_to_index(x + i, y + j)
                    .and_then(|neighbor_index| {
                        Some(
                            if self.cells[neighbor_index].state == CellState::Covered
                                && self.cells[neighbor_index].has_mine == false
                            {
                                self.uncover(x + i, y + j);
                            },
                        )
                    });
            }
        }
        BoardState::InProgress
    }

    pub fn toggle_flag(&mut self, x: i32, y: i32) -> i32 {
        let index = self.coord_to_index(x, y).unwrap();
        match self.cells[index].state {
            CellState::Flagged => {
                self.cells[index].state = CellState::Covered;
                -1
            }
            CellState::Covered => {
                self.cells[index].state = CellState::Flagged;
                1
            }
            _ => 0,
        }
    }

    pub fn uncover_bombs(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = self.coord_to_index(x, y).unwrap();
                if self.cells[index].has_mine == true {
                    self.cells[index].state = CellState::Exposed;
                }
            }
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Cell> {
        let index = self.coord_to_index(x, y)?;
        Some(self.cells[index].clone())
    }

    pub fn get_neighbors(&self, x: i32, y: i32) -> Vec<&Cell> {
        let mut neighbors = vec![];
        for (i, j) in NEIGHBOR_OFFSETS.iter() {
            if x == 0 && y == 0 {
                continue;
            }
            self.coord_to_index(x + i, y + j)
                .and_then(|index| Some(neighbors.push(&self.cells[index])));
        }
        neighbors
    }

    fn place_mine(&mut self, x: i32, y: i32) {
        let index = self.coord_to_index(x, y).unwrap();
        self.cells[index].has_mine = true;
        self.cells[index].neighboring_mines = self
            .get_neighbors(x, y)
            .iter()
            .filter(|cell| cell.has_mine)
            .collect::<Vec<&&Cell>>()
            .len() as u8;

        // Update neighbor mine counts
        for (i, j) in NEIGHBOR_OFFSETS.iter() {
            if x == 0 && y == 0 {
                continue;
            }
            self.coord_to_index(x + i, y + j)
                .and_then(|index| Some(self.cells[index].neighboring_mines += 1));
        }
    }

    /** Returns `None` if coord is out of bounds */
    fn coord_to_index(&self, x: i32, y: i32) -> Option<usize> {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            Some((x + y * self.width) as usize)
        } else {
            None
        }
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.cells
                .iter()
                .map(|cell| format!("{}", &cell))
                .collect::<Vec<String>>()
                .chunks(self.width as usize)
                .map(|chunk| chunk.join(" "))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Clone)]
struct Cell {
    state: CellState,
    has_mine: bool,
    neighboring_mines: u8,
    x: i32,
    y: i32,
}

impl Cell {
    fn new(state: CellState, has_mine: bool, neighboring_mines: u8, x: i32, y: i32) -> Self {
        Cell {
            state,
            has_mine,
            neighboring_mines,
            x,
            y,
        }
    }

    pub fn sprite_index(&self) -> usize {
        match self.state {
            CellState::Covered => 13,
            CellState::Exposed => {
                if self.has_mine {
                    10
                } else {
                    if self.neighboring_mines == 0 {
                        14
                    } else {
                        self.neighboring_mines as usize
                    }
                }
            }
            CellState::Flagged => 11,
        }
    }
}

#[derive(Clone, PartialEq)]
enum CellState {
    Covered,
    Exposed,
    Flagged,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match &self.state {
                CellState::Covered => '-',
                CellState::Exposed => {
                    if self.has_mine {
                        '%'
                    } else {
                        self.neighboring_mines
                            .to_string()
                            .chars()
                            .collect::<Vec<char>>()[0]
                    }
                }
                CellState::Flagged => 'F',
            }
        )
    }
}

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

    let state = &mut GameState {
        grid,
        spritesheet,
        total_mines: mine_count,
        total_flags: 0,
        play_state: PlayState::Unstarted,
        turns: 0,
    };
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

fn number_to_sprites(x: i32) -> Vec<u8> {
    x.to_string()
        .chars()
        .map(|digit| digit.to_string().parse::<u8>().unwrap())
        .collect::<Vec<u8>>()
}
