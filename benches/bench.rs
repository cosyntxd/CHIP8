use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};
use std::fs;
use std::{path::PathBuf, time::Duration};

fn general_bench(c: &mut Criterion) {
    for path in fs::read_dir("roms").unwrap() {
        if let Ok(entry) = path {
            if entry.path().extension().unwrap() == "ch8" {
                bench_app(c, entry.path());
            }
        }
    }
}
fn bench_app(c: &mut Criterion, path: PathBuf) {
    c.bench_function(path.to_str().unwrap(), |f| {
        f.iter_batched(
            || {
                let mut interpreter = chip8::chip8::Chip8Interpreter::new();
                interpreter.load_rom(path.clone()).unwrap();
                black_box(interpreter)
            },
            |mut interpreter| {
                for _ in 0..100_000 {
                    interpreter.execute_cycle();
                }
            },
            BatchSize::LargeInput,
        )
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(5));
    targets = general_bench
);
criterion_main!(benches);
