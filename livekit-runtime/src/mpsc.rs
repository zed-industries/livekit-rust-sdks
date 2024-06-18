///! This module implements a subset of tokio's mpsc API in terms
///! of the `async-channel` crate.
use futures::StreamExt;
use std::task::{Context, Poll};

pub fn channel<T>(size: usize) -> (Sender<T>, Receiver<T>) {
    let (tx, rx) = async_channel::bounded(size);
    (Sender(tx), Receiver(rx))
}

pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = async_channel::unbounded();
    (UnboundedSender(tx), UnboundedReceiver(rx))
}

#[derive(Clone)]
pub struct UnboundedSender<T>(async_channel::Sender<T>);
pub struct UnboundedReceiver<T>(async_channel::Receiver<T>);

#[derive(Clone)]
pub struct Sender<T>(async_channel::Sender<T>);
pub struct Receiver<T>(async_channel::Receiver<T>);

impl<T> Sender<T> {
    pub async fn send(&self, value: T) -> Result<(), error::SendError<T>> {
        self.0.send(value).await
    }

    pub fn blocking_send(&self, value: T) -> Result<(), error::SendError<T>> {
        self.0.send_blocking(value)
    }
}

impl<T> UnboundedReceiver<T> {
    pub fn close(&mut self) {
        self.0.close();
    }

    pub fn poll_recv(&mut self, cx: &mut Context) -> Poll<Option<T>> {
        self.0.poll_next_unpin(cx)
    }
}

impl<T> Receiver<T> {
    pub fn try_recv(&mut self) -> Result<T, error::TryRecvError> {
        self.0.try_recv().map_err(|error| match error {
            async_channel::TryRecvError::Empty => error::TryRecvError::Empty,
            async_channel::TryRecvError::Closed => error::TryRecvError::Disconnected,
        })
    }

    pub async fn recv(&mut self) -> Option<T> {
        self.0.next().await
    }
}

impl<T> UnboundedSender<T> {
    pub fn send(&self, value: T) -> Result<(), error::SendError<T>> {
        self.0.send_blocking(value)
    }
}

pub mod error {
    pub use async_channel::SendError;

    #[derive(Debug)]
    pub enum TryRecvError {
        Disconnected,
        Empty,
    }
}
