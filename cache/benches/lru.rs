use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion,
};

use cache::lru::LruCache;

fn insert(mut cache: LruCache<i32, &'static str>) {
    black_box(cache.insert(1000, "a"));
}

fn get(mut cache: LruCache<i32, &'static str>) {
    black_box(cache.get(&0));
}

fn new_lru() -> LruCache<i32, &'static str> {
    let mut cache = LruCache::new(1001);
    for i in 0..1000 {
        cache.insert(i, "");
    }
    cache
}

fn bench(c: &mut Criterion) {
    let cache = new_lru();

    c.bench_function("get", |b| {
        b.iter_batched(
            || cache.clone(),
            get,
            BatchSize::SmallInput,
        );
    });

    c.bench_function("insert", |b| {
        b.iter_batched(
            || cache.clone(),
            insert,
            BatchSize::SmallInput,
        );
    });

    c.bench_function("insert overflow", |b| {
        b.iter_batched(
            || {
                let mut cache = cache.clone();
                cache.insert(1001, "");
                cache
            },
            insert,
            BatchSize::SmallInput,
        );
    });

    c.bench_function("insert repeat", |b| {
        b.iter_batched(
            || cache.clone(),
            |mut cache| {
                cache.insert(1, "a");
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
