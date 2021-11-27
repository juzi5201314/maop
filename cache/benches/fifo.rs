use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, Criterion,
};

use cache::fifo::{ContainerImpl, FifoCache};

fn pop(mut cache: FifoCache<i32>) {
    black_box(cache.pop());
}

fn push(mut cache: FifoCache<i32>) {
    black_box(cache.push(0));
}

fn bench_linked_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("fifo linked_list");
    let mut cache = FifoCache::new(ContainerImpl::LinkedList, 1000);
    for i in 0..999 {
        cache.push(i);
    }

    group.bench_function("pop", |b| {
        b.iter_batched(|| cache.clone(), pop, BatchSize::SmallInput);
    });

    group.bench_function("push", |b| {
        b.iter_batched(|| cache.clone(), push, BatchSize::SmallInput);
    });

    group.bench_function("push overflow", |b| {
        b.iter_batched(
            || {
                let mut cache = cache.clone();
                cache.push(0);
                cache
            },
            push,
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

fn bench_vecdeque(c: &mut Criterion) {
    let mut group = c.benchmark_group("fifo vecdeque");
    let mut cache = FifoCache::new(ContainerImpl::VecDeque, 1000);
    for i in 0..999 {
        cache.push(i);
    }

    group.bench_function("pop", |b| {
        b.iter_batched(|| cache.clone(), pop, BatchSize::SmallInput);
    });

    group.bench_function("push", |b| {
        b.iter_batched(|| cache.clone(), push, BatchSize::SmallInput);
    });

    group.bench_function("push overflow", |b| {
        b.iter_batched(
            || {
                let mut cache = cache.clone();
                cache.push(0);
                cache
            },
            push,
            BatchSize::SmallInput,
        );
    });
    group.finish();
}

criterion_group!(benches, bench_linked_list, bench_vecdeque);
criterion_main!(benches);
