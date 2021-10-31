use std::future::Future;
use tokio::task::JoinHandle;

pub fn spawn<T>(fut: T, _name: &str) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    cfg_if::cfg_if! {
        if #[cfg(tokio_unstable)] {
            tokio::task::Builder::new()
                .name(_name)
                .spawn(fut)
        } else {
            tokio::task::spawn(fut)
        }
    }
}
