#[tokio::main(flavor = "multi_thread")]
async fn main() {
    core::run().await.unwrap();
}
