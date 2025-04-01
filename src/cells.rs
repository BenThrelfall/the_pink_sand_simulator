use std::collections::HashSet;

use rand::{seq::SliceRandom, Rng};
use CellKind::*;

use crate::point::{point, Point, FALL_SLIDE_LEFT, FALL_SLIDE_RIGHT};

#[derive(Debug, Clone)]
struct CellData {
    x_size: usize,
    y_size: usize,
    data: Vec<Cell>,
}

impl CellData {
    fn new(x_size: usize, y_size: usize) -> Self {
        Self {
            x_size,
            y_size,
            data: vec![Cell::new(Air); x_size * y_size],
        }
    }

    #[inline]
    pub fn cell_at(&self, x: usize, y: usize) -> &Cell {
        &self.data[x * self.y_size + y]
    }

    pub fn cell_at_mut(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.data[x * self.y_size + y]
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.x_size && y < self.y_size {
            self.data[x * self.y_size + y] = cell;
        }
    }

    pub fn swap_cells(&mut self, x: usize, y: usize, a: usize, b: usize) {
        self.data[a * self.y_size + b].swapped = true;
        self.data.swap(x * self.y_size + y, a * self.y_size + b);
    }

    pub fn try_swap(&mut self, x: usize, y: usize, a: usize, b: usize) -> bool {
        if self.cell_at(a, b).not_air() && self.cell_at(a, b).swapped {
            return false;
        }

        let density_diff = self.cell_at(x, y).density() - self.cell_at(a, b).density();
        if rand::rng().random_range(1..10) <= density_diff {
            self.swap_cells(x, y, a, b);
            true
        } else {
            false
        }
    }

    /// Try to swap the cell at `from` to the `offset` points (local to from) until a swap succeeds.
    pub fn multi_try_swap(&mut self, from: Point, offsets: &[Point]) {
        let (x, y) = from.tup();

        for point in offsets {
            let (a, b) = (*point + from).tup();
            if self.try_swap(x as usize, y as usize, a as usize, b as usize) {
                break;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cells {
    data: CellData,
    index: Vec<(usize, usize)>,
    skip: HashSet<(usize, usize)>,
}

impl Cells {
    pub fn new(x_size: usize, y_size: usize) -> Cells {
        let mut out = Cells {
            data: CellData::new(x_size, y_size),
            index: (1..x_size - 1)
                .flat_map(|n| (1..y_size - 1).map(move |m| (n, m)))
                .collect(),
            skip: HashSet::new(),
        };

        for x in 0..out.data.x_size {
            out.set_cell(x, out.data.y_size, Cell::new(Sand));
        }

        out
    }

    #[inline]
    pub fn cell_at(&self, x: usize, y: usize) -> &Cell {
        self.data.cell_at(x, y)
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: Cell) {
        self.data.set_cell(x, y, cell);
    }

    pub fn update_all(&mut self) {
        let Cells { data, index, skip } = self;

        index.shuffle(&mut rand::rng());
        skip.clear();

        index.iter().copied().for_each(|x| {
            if data.cell_at(x.0, x.1).is_air() {
                skip.insert(x);
            }
        });

        for x in 0..data.x_size {
            skip.insert((x, data.y_size - 1));
        }

        for point in self.index.iter().copied() {
            update_cell(data, skip, point);
        }

        //println!("{}", self.data.data.iter().filter(|x| **x).count())
    }
}

fn update_cell(data: &mut CellData, skip: &mut HashSet<(usize, usize)>, point: (usize, usize)) {
    if skip.contains(&point) {
        return;
    }

    skip.insert(point);
    let (x, y) = point;
    data.cell_at_mut(x, y).swapped = false;

    if x == 0 || x == data.x_size - 1 || y == 0 || y == data.y_size - 1 {
        return;
    }

    match data.cell_at(x, y).kind {
        Water => water_update(data, skip, x, y),
        Honey => honey_update(data, skip, x, y),
        Sand => sand_update(data, skip, x, y),
        PinkSand => pink_sand_update(data, skip, x, y),
        Air => (),
        PurpleSand => purple_sand_update(data, skip, x, y),
        BlueSand => blue_sand_update(data, skip, x, y),
    }
}

fn water_update(data: &mut CellData, skip: &mut HashSet<(usize, usize)>, x: usize, y: usize) {
    update_cell(data, skip, (x, y + 1));
    update_cell(data, skip, (x + 1, y));
    update_cell(data, skip, (x - 1, y));

    let pref = rand::rng().random_bool(0.5);
    if data.cell_at(x, y + 1).is_air() {
        data.set_cell(x, y + 1, Cell::new(Water));
    } else if pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(Water));
    } else if data.cell_at(x - 1, y + 1).is_air() {
        data.set_cell(x - 1, y + 1, Cell::new(Water));
    } else if pref && data.cell_at(x + 1, y).is_air() {
        data.set_cell(x + 1, y, Cell::new(Water));
    } else if data.cell_at(x - 1, y).is_air() {
        data.set_cell(x - 1, y, Cell::new(Water));
    } else if data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(Water));
    } else if data.cell_at(x + 1, y).is_air() {
        data.set_cell(x + 1, y, Cell::new(Water));
    } else {
        return;
    }

    data.set_cell(x, y, Cell::new(Air));
}
fn honey_update(data: &mut CellData, skip: &mut HashSet<(usize, usize)>, x: usize, y: usize) {
    update_cell(data, skip, (x, y + 1));

    let pref = rand::rng().random_bool(0.5);

    let targets = if pref {
        FALL_SLIDE_RIGHT
    }
    else {
        FALL_SLIDE_LEFT
    };

    data.multi_try_swap(point(x as i32, y as i32), &targets);
}

fn sand_update(data: &mut CellData, skip: &mut HashSet<(usize, usize)>, x: usize, y: usize) {
    update_cell(data, skip, (x, y + 1));

    let pref = rand::rng().random_bool(0.5);

    let _ = data.try_swap(x, y, x, y + 1)
        || pref && data.try_swap(x, y, x + 1, y + 1)
        || data.try_swap(x, y, x - 1, y + 1)
        || data.try_swap(x, y, x + 1, y + 1);
}
fn pink_sand_update(data: &mut CellData, _skip: &mut HashSet<(usize, usize)>, x: usize, y: usize) {
    let pref = rand::rng().random_bool(0.5);

    if data.cell_at(x, y + 1).is_air() {
        data.set_cell(x, y + 1, Cell::new(PinkSand));
    } else if pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(PinkSand));
    } else if data.cell_at(x - 1, y + 1).is_air() {
        data.set_cell(x - 1, y + 1, Cell::new(PinkSand));
    } else if !pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(PinkSand));
    } else {
        return;
    }

    data.set_cell(x, y, Cell::new(Air));
}
fn purple_sand_update(data: &mut CellData, skip: &mut HashSet<(usize, usize)>, x: usize, y: usize) {
    update_cell(data, skip, (x, y - 1));

    let pref = rand::rng().random_bool(0.5);

    if data.cell_at(x, y + 1).is_air() {
        data.set_cell(x, y + 1, Cell::new(PurpleSand));
    } else if pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(PurpleSand));
    } else if data.cell_at(x - 1, y + 1).is_air() {
        data.set_cell(x - 1, y + 1, Cell::new(PurpleSand));
    } else if !pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(PurpleSand));
    } else if data.cell_at(x, y - 1).is_air() {
        data.set_cell(x, y - 1, Cell::new(PurpleSand));
    } else {
        return;
    }

    data.set_cell(x, y, Cell::new(Air));
}
fn blue_sand_update(data: &mut CellData, skip: &mut HashSet<(usize, usize)>, x: usize, y: usize) {
    update_cell(data, skip, (x, y + 1));

    let pref = rand::rng().random_bool(0.5);

    if data.cell_at(x, y + 1).is_air() {
        data.set_cell(x, y + 1, Cell::new(BlueSand));
    } else if pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(BlueSand));
    } else if data.cell_at(x - 1, y + 1).is_air() {
        data.set_cell(x - 1, y + 1, Cell::new(BlueSand));
    } else if !pref && data.cell_at(x + 1, y + 1).is_air() {
        data.set_cell(x + 1, y + 1, Cell::new(BlueSand));
    } else if data.cell_at(x, y - 1).is_air() {
        data.set_cell(x, y - 1, Cell::new(BlueSand));
    } else {
        return;
    }

    data.set_cell(x, y, Cell::new(Air));
}
#[derive(Debug, Clone)]
pub struct Cell {
    swapped: bool,
    kind: CellKind,
}

impl Cell {
    pub fn new(kind: CellKind) -> Cell {
        Cell {
            kind,
            swapped: false,
        }
    }

    pub fn not_air(&self) -> bool {
        match self.kind {
            Air => false,
            _ => true,
        }
    }

    pub fn is_air(&self) -> bool {
        match self.kind {
            Air => true,
            _ => false,
        }
    }

    pub fn is(&self, cells: &[CellKind]) -> bool {
        cells.contains(&self.kind)
    }

    pub fn colour(&self) -> [u8; 4] {
        match self.kind {
            Water => [30, 76, 200, 255],
            Honey => [140, 90, 50, 255],
            PinkSand => [160, 80, 110, 255],
            Sand => [200, 100, 50, 255],
            Air => [10, 10, 10, 255],
            PurpleSand => [120, 80, 180, 255],
            BlueSand => [90, 70, 210, 255],
        }
    }

    pub fn density(&self) -> i32 {
        match self.kind {
            PurpleSand => 30,
            BlueSand => 30,
            Water => 25,
            Honey => 28,
            Sand => 30,
            PinkSand => 30,
            Air => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellKind {
    PurpleSand,
    BlueSand,
    Water,
    Honey,
    Sand,
    PinkSand,
    Air,
}
