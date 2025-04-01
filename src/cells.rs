use std::collections::HashSet;

use rand::{seq::SliceRandom, Rng};
use CellKind::*;

use crate::point::{
    point, Point, DOWN, FALL_SLIDE_LEFT, FALL_SLIDE_RIGHT, FALL_TUMBLE_LEFT, FALL_TUMBLE_RIGHT,
    LEFT, RIGHT, UP,
};

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
    pub fn cell_at(&self, point: Point) -> &Cell {
        &self.data[point.index(self.y_size)]
    }

    pub fn cell_at_mut(&mut self, point: Point) -> &mut Cell {
        &mut self.data[point.index(self.y_size)]
    }

    pub fn set_cell(&mut self, point: Point, cell: Cell) {
        let (x, y) = point.utup();
        if x < self.x_size && y < self.y_size {
            self.data[x * self.y_size + y] = cell;
        }
    }

    pub fn swap_cells(&mut self, from: Point, to: Point) {
        self.data[to.index(self.y_size)].swapped = true;
        self.data
            .swap(from.index(self.y_size), to.index(self.y_size));
    }

    pub fn try_swap(&mut self, from: Point, to: Point) -> bool {
        if self.cell_at(to).not_air() && self.cell_at(to).swapped {
            return false;
        }

        let density_diff = self.cell_at(from).density() - self.cell_at(to).density();
        if rand::rng().random_range(1..10) <= density_diff {
            self.swap_cells(from, to);
            true
        } else {
            false
        }
    }

    /// Try to swap the cell at `from` to the `offset` points (local to from) until a swap succeeds.
    pub fn multi_try_swap(&mut self, from: Point, offsets: &[Point]) -> bool {
        for point in offsets {
            let to = *point + from;
            if self.try_swap(from, to) {
                return true;
            }
        }

        false
    }
}

#[derive(Debug, Clone)]
pub struct Cells {
    data: CellData,
    index: Vec<Point>,
    skip: HashSet<Point>,
}

impl Cells {
    pub fn new(x_size: usize, y_size: usize) -> Cells {
        let out = Cells {
            data: CellData::new(x_size, y_size),
            index: (1..x_size as i32 - 1)
                .flat_map(|n| (1..y_size as i32 - 1).map(move |m| point(n, m)))
                .collect(),
            skip: HashSet::new(),
        };

        out
    }

    #[inline]
    pub fn cell_at(&self, point: Point) -> &Cell {
        self.data.cell_at(point)
    }

    pub fn set_cell(&mut self, point: Point, cell: Cell) {
        self.data.set_cell(point, cell);
    }

    pub fn update_all(&mut self) {
        let Cells { data, index, skip } = self;

        index.shuffle(&mut rand::rng());
        skip.clear();

        index.iter().copied().for_each(|x| {
            if data.cell_at(x).is_air() {
                skip.insert(x);
            }
        });

        for x in 0..data.x_size {
            skip.insert(point(x as i32, 0));
        }

        for point in self.index.iter().copied() {
            update_cell(data, skip, point);
        }

        //println!("{}", self.data.data.iter().filter(|x| **x).count())
    }
}

fn update_cell(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    if skip.contains(&point) {
        return;
    }

    skip.insert(point);
    data.cell_at_mut(point).swapped = false;

    match data.cell_at(point).kind {
        Water => water_update(data, skip, point),
        Honey => honey_update(data, skip, point),
        Sand => sand_update(data, skip, point),
        PinkSand => pink_sand_update(data, skip, point),
        Air => (),
        PurpleSand => purple_sand_update(data, skip, point),
        BlueSand => blue_sand_update(data, skip, point),
        Bedrock => (),
    }
}

fn water_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + DOWN);
    update_cell(data, skip, point + RIGHT);
    update_cell(data, skip, point + LEFT);

    let pref = rand::rng().random_bool(0.5);

    let targets = if pref {
        FALL_SLIDE_RIGHT
    } else {
        FALL_SLIDE_LEFT
    };

    data.multi_try_swap(point, &targets);
}
fn honey_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + DOWN);

    let pref = rand::rng().random_bool(0.5);

    let targets = if pref {
        FALL_SLIDE_RIGHT
    } else {
        FALL_SLIDE_LEFT
    };

    data.multi_try_swap(point, &targets);
}

fn sand_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + DOWN);

    let pref = rand::rng().random_bool(0.5);

    if pref {
        data.multi_try_swap(point, &FALL_TUMBLE_RIGHT);
    } else {
        data.multi_try_swap(point, &FALL_TUMBLE_LEFT);
    }
}

fn pink_sand_update(data: &mut CellData, _skip: &mut HashSet<Point>, point: Point) {
    let pref = rand::rng().random_bool(0.5);

    if pref {
        data.multi_try_swap(point, &FALL_TUMBLE_RIGHT);
    } else {
        data.multi_try_swap(point, &FALL_TUMBLE_LEFT);
    }
}

fn purple_sand_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + UP);

    let pref = rand::rng().random_bool(0.5);

    if pref {
        let _ = data.multi_try_swap(point, &FALL_TUMBLE_RIGHT) || data.try_swap(point, point + UP);
    } else {
        let _ = data.multi_try_swap(point, &FALL_TUMBLE_LEFT) || data.try_swap(point, point + UP);
    }
}
fn blue_sand_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + DOWN);

    let pref = rand::rng().random_bool(0.5);

    let _ = pref && data.multi_try_swap(point, &FALL_TUMBLE_RIGHT)
        || data.multi_try_swap(point, &FALL_TUMBLE_LEFT)
        || data.try_swap(point, point + UP);
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
            Air => [200, 200, 235, 255],
            //Air => [10, 10, 10, 255],
            PurpleSand => [120, 80, 180, 255],
            BlueSand => [90, 70, 210, 255],
            Bedrock => [13, 39, 20, 255],
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
            Bedrock => 500,
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
    Bedrock,
}
