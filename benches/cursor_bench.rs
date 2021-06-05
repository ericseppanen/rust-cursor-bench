use bytes::{BufMut, BytesMut};
use criterion::{black_box, criterion_group, criterion_main, profiler::Profiler, Criterion};
use rust_cursor_bench::{MyStruct, WriterVec, serialize_it};
use pprof::ProfilerGuard;
use std::{fs::File, io::Cursor, path::Path};

// Ick.
type Freq = std::os::raw::c_int;
pub struct FlamegraphProfiler<'a> {
    frequency: Freq,
    active_profiler: Option<ProfilerGuard<'a>>,
}

impl<'a> FlamegraphProfiler<'a> {
    #[allow(dead_code)]
    pub fn new(frequency: Freq) -> Self {
        FlamegraphProfiler {
            frequency,
            active_profiler: None,
        }
    }
}

impl<'a> Profiler for FlamegraphProfiler<'a> {
    fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
        self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
    }

    fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
        std::fs::create_dir_all(benchmark_dir).unwrap();
        let flamegraph_path = benchmark_dir.join("flamegraph.svg");
        let flamegraph_file = File::create(&flamegraph_path)
            .expect("File system error while creating flamegraph.svg");
        if let Some(profiler) = self.active_profiler.take() {
            profiler
                .report()
                .build()
                .unwrap()
                .flamegraph(flamegraph_file)
                .expect("Error writing flamegraph");
        }
    }
}

const ENCODED_SIZE: usize = 16;
const NUM_CHUNKS: usize = 2048;
const BUFFER_SIZE: usize = NUM_CHUNKS * ENCODED_SIZE;

fn do_cursor_vec() {
    let mut c = Cursor::new(Vec::<u8>::with_capacity(BUFFER_SIZE));

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut c);
    }

    let v = c.into_inner();
    assert_eq!(v.len(), BUFFER_SIZE);
    let s: u8 = v.iter().sum();
    assert_eq!(s, 24);
}

fn do_cursor_vec_ref() {
    let mut v = Vec::<u8>::with_capacity(BUFFER_SIZE);
    let mut c = Cursor::new(&mut v);

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut c);
    }

    assert_eq!(v.len(), BUFFER_SIZE);
    let s: u8 = v.iter().sum();
    assert_eq!(s, 24);
}

fn do_cursor_slice() {
    let mut b = [0; BUFFER_SIZE];
    let mut c = Cursor::new(&mut b[..]);

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut c);
    }

    let s: u8 = b.iter().sum();
    assert_eq!(s, 24);
}

fn do_cursor_box() {
    let b: Box<[u8]> = Box::new([0; BUFFER_SIZE]);
    let mut c = Cursor::new(b);

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut c);
    }

    let b = c.into_inner();
    let s: u8 = b.iter().sum();
    assert_eq!(s, 24);
}

fn do_bytesmut() {
    let mut b = BytesMut::with_capacity(BUFFER_SIZE).writer();

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut b);
    
    }

    let v = b.into_inner();
    assert_eq!(v.len(), BUFFER_SIZE);

    let s: u8 = v[..].iter().sum();
    assert_eq!(s, 24);
}

fn do_writervec() {
    let mut b = WriterVec::with_capacity(BUFFER_SIZE);

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut b);
    
    }

    let v = b.into_inner();
    assert_eq!(v.len(), BUFFER_SIZE);

    let s: u8 = v.iter().sum();
    assert_eq!(s, 24);
}

fn do_array() {
    let mut b = [0; BUFFER_SIZE];
    let mut cursor = &mut b[..];

    for ii in 0..NUM_CHUNKS {
        // Conceal the source of the data, to avoid optimizing it away
        let test_struct = black_box(MyStruct::new(ii));
        serialize_it(&test_struct, &mut cursor);
    
    }

    //assert_eq!(cursor.position(), BUFFER_SIZE);

    let s: u8 = b.iter().sum();
    assert_eq!(s, 24);
}

fn bench_cursors(c: &mut Criterion) {
    let mut group = c.benchmark_group("-");
    group.bench_function("Cursor<Vec<u8>>", |b| b.iter(do_cursor_vec));
    group.bench_function("Cursor<&mut Vec<u8>>", |b| b.iter(do_cursor_vec_ref));
    group.bench_function("WriterVec", |b| b.iter(do_writervec));
    group.bench_function("BytesMut", |b| b.iter(do_bytesmut));
    group.bench_function("Cursor<&mut [u8]>", |b| b.iter(do_cursor_slice));
    group.bench_function("Cursor<Box<[u8]>>", |b| b.iter(do_cursor_box));
    group.bench_function("&mut [u8]", |b| b.iter(do_array));
}

criterion_group! {
    name = benches;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
    targets = bench_cursors
}
criterion_main!(benches);
