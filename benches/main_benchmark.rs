use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use the_pink_sand_simulator::{
    cells::{CellKind::*, Cells, cell},
    point::point,
};

fn simple_benchmarks(c: &mut Criterion) {
    c.bench_function("Empty World", |b| b.iter(|| black_box(empty_world())));
    c.bench_function("Single Water", |b| b.iter(|| black_box(single_water())));
}

fn complex_benchmarks(c: &mut Criterion) {
    c.bench_function("Water Spawner", |b| b.iter(|| black_box(water_spawner())));
    c.bench_function("Water Plinko", |b| b.iter(|| black_box(water_plinko())));
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

fn water_spawner() {
    let mut cells = Cells::new();

    for _ in 0..1000 {
        cells.set_cell(point(0, 0), cell(Water));
        cells.update_all();
    }
}

fn single_water() {
    let mut cells = Cells::new();

    cells.set_cell(point(0, 0), cell(Water));

    for _ in 0..10000 {
        cells.update_all();
    }
}

fn empty_world() {
    let mut cells = Cells::new();

    for _ in 0..10000 {
        cells.update_all();
    }
}

criterion_group!(simple, simple_benchmarks);
criterion_group!(
    name = complex; 
    config = Criterion::default().sample_size(10);
    targets = complex_benchmarks
);

criterion_main!(simple, complex);
