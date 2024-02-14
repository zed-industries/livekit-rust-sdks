use crate::runtime::Dispatcher;

pub struct TokioDispatcher {
    runtime: tokio::runtime::Handle
}

impl TokioDispatcher {
    pub fn new() -> Self {
        TokioDispatcher {
            runtime: tokio::runtime::Handle::current()
        }
    }
}

impl Dispatcher for TokioDispatcher {
    fn dispatch(&self, runnable: async_task::Runnable) {
        self.runtime.spawn(async move {
            runnable.run();
        });
    }

    fn dispatch_after(&self, duration: std::time::Duration, runnable: async_task::Runnable) {
        self.runtime.spawn(async move {
            tokio::time::sleep(duration).await;
            runnable.run();
        });
    }
}