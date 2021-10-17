use crossfire::mpsc::{TxUnbounded, unbounded_future, RxUnbounded};
use tokio::sync::Mutex;

pub struct Notify {
    targets: Mutex<Vec<Target>>,
}

impl Notify {
    pub fn new() -> Self {
        Notify {
            targets: Mutex::new(Vec::new())
        }
    }

    pub async fn notify(&self) {
        let mut targets = self.targets.lock().await;
        targets.sort_by(|t1, t2| t1.priority.cmp(&t2.priority).reverse());

        for target in targets.iter() {
            target.tx.send(()).ok();
            target.rx.recv().await.ok();
        }
    }

    pub async fn register(&self, priority: usize) -> WaitHandle {
        let (tx, rx) = unbounded_future();
        let (tx2, rx2) = unbounded_future();
        self.targets.lock().await.push(Target {
            priority,
            tx,
            rx: rx2
        });
        WaitHandle {
            tx: tx2,
            rx
        }
    }
}

struct Target {
    priority: usize,
    tx: TxUnbounded<()>,
    rx: RxUnbounded<()>,
}

pub struct WaitHandle {
    tx: TxUnbounded<()>,
    rx: RxUnbounded<()>,
}

impl WaitHandle {
    pub async fn wait(&self) -> Response {
        self.rx.recv().await.ok();
        Response(self.tx.clone())
    }
}

pub struct Response(TxUnbounded<()>);

impl Response {
    pub fn ready(self) {
        self.0.send(()).ok();
    }
}
