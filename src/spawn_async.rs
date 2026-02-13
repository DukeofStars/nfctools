use std::future::Future;

pub fn spawn_async<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> impl Future<Output = T> {
    let (sender, receiver) = futures::channel::oneshot::channel();

    std::thread::spawn(move || {
        let res = f();
        // ignore error if receiver is dropped
        let _ = sender.send(res);
    });

    async move { receiver.await.expect("Thread panicked or sender dropped") }
}
