#[cfg(feature = "snmalloc-rs")]
#[global_allocator]
static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    core::run().await.unwrap();
}
