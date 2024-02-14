use core::sync::atomic::Ordering;
use std::{
    cell::OnceCell,
    pin::Pin,
    sync::{atomic::AtomicBool, Arc, OnceLock},
    task::{Context, Poll, Wake, Waker},
    time::Duration,
};

use async_task::Runnable;
use futures_util::{pin_mut, Future};
// use parking::Unparker;

// Initial, adds a generic to every type
trait Runtime {
    type JoinHandle;
}

//sqlx, closed set, inflexible
enum JoinHandle {
    #[cfg(tokio)]
    TokioJoinHandle(),
    #[cfg(async_std)]
    AsyncStdJoinHandle(),
}
// type JoinHandle =

/// Task is a primitive that allows work to happen in the background.
///
/// It implements [`Future`] so you can `.await` on it.
///
/// If you drop a task it will be cancelled immediately. Calling [`Task::detach`] allows
/// the task to continue running, but with no way to return a value.
#[must_use]
#[derive(Debug)]
pub enum Task<T> {
    /// A task that is ready to return a value
    Ready(Option<T>),

    /// A task that is currently running.
    Spawned(async_task::Task<T>),
}

impl<T> Task<T> {
    /// Creates a new task that will resolve with the value
    pub fn ready(val: T) -> Self {
        Task::Ready(Some(val))
    }

    /// Detaching a task runs it to completion in the background
    pub fn detach(self) {
        match self {
            Task::Ready(_) => {}
            Task::Spawned(task) => task.detach(),
        }
    }
}

impl<T> Future for Task<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        match unsafe { self.get_unchecked_mut() } {
            Task::Ready(val) => Poll::Ready(val.take().unwrap()),
            Task::Spawned(task) => Pin::new(task).poll(cx),
        }
    }
}

pub trait Dispatcher: 'static + Send + Sync {
    fn dispatch(&self, runnable: Runnable);
    fn dispatch_after(&self, duration: Duration, runnable: Runnable);
}

static GLOBAL_DISPATCHER: OnceLock<Arc<dyn Dispatcher>> = OnceLock::new();

pub fn set_global_dispatcher(dispatcher: impl Dispatcher) {
    GLOBAL_DISPATCHER.get_or_init(|| Arc::new(dispatcher));
}

pub fn timer(duration: Duration) -> Task<()> {
    let dispatcher = GLOBAL_DISPATCHER.get().expect("call set_global_dispatcher first").clone();

    let (runnable, task) = async_task::spawn(async move {}, {
        move |runnable| dispatcher.dispatch_after(duration, runnable)
    });
    runnable.schedule();
    Task::Spawned(task)
}

pub fn spawn<R>(future: impl Future<Output = R> + Send + 'static) -> Task<R>
where
    R: Send + 'static,
{
    let dispatcher = GLOBAL_DISPATCHER.get().expect("call set_global_dispatcher first").clone();

    let (runnable, task) = async_task::spawn(future, move |runnable| dispatcher.dispatch(runnable));
    runnable.schedule();
    Task::Spawned(task)
}

// Takeaways:
// - We need ot figure out what to do with livekit options:
//  -
// - Schedule a meeting to talk about the executor in LiveKit:
//  - Why did they choose tokio?
//  - Is there a way to work with us and their applications?
//      - with Few or no downsides
//  - To demonstrate this, we want to make sure that we can:
//      - run a tokio dispatcher
//      - Able to use some or replace of their sync primitives
//      - Make sure tungstenite works -> switch to async_tungestenite
// -
//
//
// Solution options:
//  - Stick with a portable subset of the livekit Swift SDK,
//    and still do the platform bindings ourselves

struct TokioDispatcher {
    handle: tokio::runtime::Handle,
}

struct RunnableFuture {
    runnable: Runnable,
}

impl Future for RunnableFuture {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.runnable.run() {
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}

impl Dispatcher for TokioDispatcher {
    fn dispatch(&self, runnable: Runnable) {
        todo!()
    }

    fn dispatch_after(&self, duration: Duration, runnable: Runnable) {
        todo!()
    }
}
