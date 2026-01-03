use the_pink_sand_simulator::{
    cells::{CellKind::*, Cells, cell},
    point::point,
};

fn main() {
    water_plinko();
}

fn water_plinko() {
    let mut cells = Cells::new();

    for y in 10..100 {
        for x in -50..50 {
            let offset = y % 2;
            cells.set_cell(point(2 * x + offset, y * -10), cell(Bedrock));
        }
    }

    for _ in 0..1000 {
        for x in -1..=1 {
            cells.set_cell(point(x, 0), cell(Water));
        }

        cells.update_all();
    }
}
