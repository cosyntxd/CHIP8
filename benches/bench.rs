use std::path::PathBuf;

use chip8::chip8::{HEIGHT, WIDTH};
use criterion::{black_box, criterion_group, criterion_main, BatchSize, Criterion};

fn general_bench(c: &mut Criterion) {
    bench_app(c, "Space Invaders [David Winter].ch8");
}
fn bench_app(c: &mut Criterion, name: &str) {
    let path = PathBuf::new().join("roms").join(name);
    let mut pixels = vec![0;HEIGHT*WIDTH*4];
    c.bench_function(&format!("{name}  |  100k iterations"), |f| {
        f.iter_batched(|| {
            let mut interpreter = chip8::chip8::Chip8Interpreter::new();
            interpreter.load_rom(path.clone()).unwrap();
            black_box(interpreter)
        }, |mut interpreter| {
            for _ in 0..100_000 {
                interpreter.execute_cycle(&mut pixels);
            }
        }, BatchSize::LargeInput) 
    });
}

criterion_group!(benches, general_bench);
criterion_main!(benches);