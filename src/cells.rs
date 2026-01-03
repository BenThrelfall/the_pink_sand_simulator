use std::collections::{HashMap, HashSet};

use rand::Rng;
use CellKind::*;

use crate::point::{
    Point, CLOSED_NEIGHBOURS, DOWN, FALL_SLIDE_LEFT, FALL_SLIDE_RIGHT, FALL_TUMBLE_LEFT, FALL_TUMBLE_RIGHT, LEFT, RIGHT, RISE_SLIDE_LEFT, RISE_SLIDE_RIGHT, SLIDE_LEFT, SLIDE_RIGHT, UP
};

const GLOBAL_AIR: Cell = Cell {
    swapped: false,
    kind: Air,
};

#[derive(Debug, Clone)]
struct CellData {
    data: HashMap<Point, Cell>,
    next_updates: HashSet<Point>,
}

impl CellData {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            next_updates: HashSet::new(),
        }
    }

    #[inline]
    pub fn cell_at(&self, point: Point) -> &Cell {
        self.data.get(&point).unwrap_or(&GLOBAL_AIR)
    }

    pub fn cell_at_mut(&mut self, point: Point) -> &mut Cell {
        self.data
            .get_mut(&point)
            .expect("You must know there is a cell here to use this method")
    }

    pub fn set_cell(&mut self, point: Point, cell: Cell) {
        self.data.insert(point, cell);
        self.changed(point);
    }

    pub fn swap_cells(&mut self, from: Point, to: Point) {
        let from_cell = self.data.remove(&from).unwrap();
        let maybe_to_cell = self.data.remove(&to);

        match maybe_to_cell {
            Some(to_cell) => {
                self.data.insert(from, to_cell);
                self.cell_at_mut(from).swapped = true;
            }
            None => (),
        };

        self.data.insert(to, from_cell);
        self.changed(to);
        self.changed(from);
    }

    pub fn awaken(&mut self, point: Point) -> bool {
        self.next_updates.insert(point);
        true
    }

    pub fn changed(&mut self, point: Point) -> bool {
        for offset in CLOSED_NEIGHBOURS {
            self.next_updates.insert(point + offset);
        } 
        true
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
    skip: HashSet<Point>,
    current_updates: HashSet<Point>,
}

impl Cells {
    pub fn new() -> Cells {
        let out = Cells {
            data: CellData::new(),
            skip: HashSet::new(),
            current_updates: HashSet::new(),
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
        let Cells { data, skip, current_updates } = self;

        std::mem::swap(current_updates, &mut data.next_updates);
        data.next_updates.clear();

        //updates.shuffle(&mut rand::rng());

        skip.clear();

        for point in current_updates.iter().copied() {
            update_cell(data, skip, point);
        }
    }
}

fn update_cell(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    if skip.contains(&point) {
        return;
    }

    skip.insert(point);

    if !data.data.contains_key(&point) {
        return;
    }

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
        Hydrogen => hydrogen_update(data, skip, point),
    }
}

fn hydrogen_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + UP);
    update_cell(data, skip, point + RIGHT);
    update_cell(data, skip, point + LEFT);

    let pref = rand::rng().random_range(1..=10);

    let targets = match pref {
        1..=3 => &RISE_SLIDE_LEFT,
        4..=6 => &RISE_SLIDE_RIGHT,
        7 => SLIDE_LEFT.as_slice(),
        8 => &SLIDE_RIGHT,
        9 => &FALL_SLIDE_LEFT,
        10 => &FALL_SLIDE_RIGHT,
        _ => panic!(),
    };

    let _ = data.multi_try_swap(point, targets) || data.awaken(point);
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

    let _ = data.multi_try_swap(point, &targets) || data.awaken(point);
}
fn honey_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + DOWN);

    let pref = rand::rng().random_bool(0.5);

    let targets = if pref {
        FALL_SLIDE_RIGHT
    } else {
        FALL_SLIDE_LEFT
    };

    if !data.multi_try_swap(point, &targets) {
        data.awaken(point);
    }
}

fn sand_update(data: &mut CellData, skip: &mut HashSet<Point>, point: Point) {
    update_cell(data, skip, point + DOWN);

    let pref = rand::rng().random_bool(0.5);

    let _ = pref && data.multi_try_swap(point, &FALL_TUMBLE_RIGHT)
        || data.multi_try_swap(point, &FALL_TUMBLE_LEFT)
        || data.awaken(point);
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

pub fn cell(kind: CellKind) -> Cell {
    Cell::new(kind)
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
            PurpleSand => [120, 80, 180, 255],
            BlueSand => [90, 70, 210, 255],
            Bedrock => [13, 39, 20, 255],
            Hydrogen => [230, 230, 230, 255],
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
            Hydrogen => 5,
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
    Hydrogen,
}
