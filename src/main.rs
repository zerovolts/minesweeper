use ggez::{
    conf::{WindowMode, WindowSetup},
    event::{self, EventHandler},
    graphics::{self, Color, DrawParam, FilterMode, Image},
    mint::{Point2, Vector2},
    Context, ContextBuilder, GameError, GameResult,
};

use std::{fmt, path::Path};

struct GameState {
    grid: Grid,
    spritesheet: Vec<Image>,
}

impl EventHandler for GameState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::new(60. / 255., 50. / 255., 83. / 255., 1.));

        for y in 0..self.grid.height {
            for x in 0..self.grid.width {
                let sprite_params = DrawParam::new()
                    .dest(Point2 {
                        x: x as f32 * 16.,
                        y: y as f32 * 16.,
                    })
                    .scale(Vector2 { x: 2., y: 2. });
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

    pub fn uncover(&mut self, x: i32, y: i32) {
        let index = self.coord_to_index(x, y).unwrap();
        self.cells[index].state = CellState::Exposed;

        // if the cell has no adjacent mines, uncover adjacent cells without adjacent mines
        if self.cells[index].neighboring_mines == 0 {
            for (i, j) in NEIGHBOR_OFFSETS.iter() {
                let neighbor_index = self.coord_to_index(x + i, y + j).unwrap();
                if self.cells[neighbor_index].state == CellState::Covered
                    && self.cells[neighbor_index].neighboring_mines == 0
                {
                    self.uncover(x + i, y + j);
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

    pub fn set_neighbors(&mut self, x: i32, y: i32, f: &dyn Fn(i32, i32, &mut Cell)) {
        for i in -1..2 {
            for j in -1..2 {
                if x == 0 && y == 0 {
                    continue;
                }
                self.coord_to_index(x + i, y + j)
                    .and_then(|index| Some(f(x + i, y + i, &mut self.cells[index])));
            }
        }
    }

    pub fn set(&mut self, x: i32, y: i32, cell: Cell) -> Option<Cell> {
        let index = self.coord_to_index(x, y)?;
        self.cells[index] = cell.clone();
        Some(cell)
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
        self.set_neighbors(x, y, &|_, _, cell: &mut Cell| cell.neighboring_mines += 1);
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
                    self.neighboring_mines as usize
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
        .window_mode(WindowMode::default().dimensions(640., 640.))
        .add_resource_path("assets")
        .build()
        .unwrap();
    let spritesheet = load_spritesheet(ctx, "/minesweeper.png", 8, 8, 4)?;

    let width = 32;
    let height = 32;
    let mut grid = Grid::new(width, height);
    for x in 0..width {
        for y in 0..height {
            if rand::random::<f32>() > 0.7 {
                grid.place_mine(x, y);
            }
        }
    }
    grid.get_neighbors(4, 4);
    grid.uncover(3, 3);
    grid.uncover(6, 8);
    grid.uncover(2, 12);
    grid.uncover(24, 2);
    grid.uncover(16, 8);
    grid.uncover(19, 20);
    println!("{}", grid);

    let state = &mut GameState { grid, spritesheet };
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
