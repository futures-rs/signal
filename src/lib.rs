use std::{
    future::Future,
    sync::{Arc, Mutex},
    task::{Poll, Waker},
};

struct Condition<Output> {
    waker: Option<Waker>,
    output: Option<Output>,
}

impl<Output> Condition<Output> {
    pub fn poll(&mut self, waker: std::task::Waker) -> Poll<Output> {
        if let Some(output) = self.output.take() {
            return Poll::Ready(output);
        }

        self.waker = Some(waker);

        Poll::Pending
    }

    pub fn ready(&mut self, output: Output) {
        assert!(self.output.is_none(), "call ready function twice");
        self.output = Some(output);

        if let Some(waker) = self.waker.take() {
            waker.wake_by_ref();
        }
    }
}

pub struct Signal<Output> {
    cond: Arc<Mutex<Condition<Output>>>,
}

impl<Output> Future for Signal<Output> {
    type Output = Output;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.cond.lock().unwrap().poll(cx.waker().clone())
    }
}

#[derive(Clone)]
pub struct Sender<Output> {
    cond: Arc<Mutex<Condition<Output>>>,
}

impl<Output> Sender<Output> {
    pub fn ready(&mut self, output: Output) {
        self.cond.lock().unwrap().ready(output)
    }
}

/// Create new condition
pub fn cond<Output>() -> (Signal<Output>, Sender<Output>) {
    let cond = Arc::new(Mutex::new(Condition {
        waker: Default::default(),
        output: Default::default(),
    }));

    (Signal { cond: cond.clone() }, Sender { cond })
}

#[cfg(test)]
mod tests {
    use crate::cond;

    #[async_std::test]
    async fn test_cond() {
        let (sig, mut sender) = cond();

        async_std::task::spawn(async move {
            sender.ready(1);
        });

        assert_eq!(sig.await, 1);
    }
}
