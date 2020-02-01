use std::fmt;

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

fn main() {
    let width = 16;
    let height = 16;
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
    println!("{}", grid);
}
