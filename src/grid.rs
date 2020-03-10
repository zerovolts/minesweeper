use ggez::{
    conf::{WindowMode, WindowSetup},
    event,
    graphics::{FilterMode, Image},
    Context, ContextBuilder, GameError,
};

use std::{fmt, path::Path, time::Duration};

pub enum BoardState {
    InProgress,
    Cleared,
    Detonated,
}

pub struct Grid {
    cells: Vec<Cell>,
    pub width: i32,
    pub height: i32,
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

    pub fn place_mine(&mut self, x: i32, y: i32) {
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
pub struct Cell {
    state: CellState,
    pub has_mine: bool,
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
