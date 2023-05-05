//! # Mini Executor
//!
//! A minimal task executor that runs a single future to completion.
//!
//! This executor is for educational purposes and is not meant for production use.
//! For a more complete and efficient executor, consider using [Tokio](https://crates.io/crates/tokio) or [async-std](https://crates.io/crates/async-std).

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// `MiniExecutor` is a minimal task executor that runs a single future to completion.
///
/// It is meant to be used for educational purposes to demonstrate how an executor works at a basic level.
pub struct MiniExecutor {
   /// A mutex containing an optional pinned boxed future.
    ///
    /// The mutex is used to ensure safe access to the future across threads.
    future: Mutex<Option<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>>,
}

impl MiniExecutor {
    /// Create a new `MiniExecutor` with the given future.
    ///
    /// # Examples
    ///
    /// ```
    /// use mini_executor::MiniExecutor;
    ///
    /// let executor = MiniExecutor::new(async {
    ///     println!("Hello from the future!");
    /// });
    /// ```
    pub fn new<F>(future: F) -> Arc<Self>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        Arc::new(Self {
            future: Mutex::new(Some(Box::pin(future))),
        })
    }

    /// Run the `MiniExecutor` to completion.
    ///
    /// This method will block the current thread until the future has completed.
    ///
    /// # Examples
    ///
    /// ```
    /// use mini_executor::MiniExecutor;
    ///
    /// let executor = MiniExecutor::new(async {
    ///     println!("Hello from the future!");
    /// });
    ///
    /// executor.run();
    /// ```
    pub fn run(self: Arc<Self>) {
        let waker = MiniExecutor::into_waker(self.clone());
        let mut context = Context::from_waker(&waker);

        // Poll the future until it's completed.
        loop {
            let mut future = self.future.lock().unwrap();
            if let Some(fut) = future.as_mut() {
                match fut.as_mut().poll(&mut context) {
                    Poll::Ready(_) => break,
                    Poll::Pending => continue,
                }
            } else {
                break;
            }
        }
    }

    /// Create a custom Waker for the `MiniExecutor`.
    ///
    /// This function generates a Waker that can be used to wake up the executor when the future is ready to make progress.
    fn into_waker(executor: Arc<Self>) -> Waker {
        let raw_waker = RawWaker::new(Arc::into_raw(executor.clone()).cast::<()>(), &VTABLE);
        unsafe { Waker::from_raw(raw_waker) }
    }
}

// The vtable for creating a custom waker for the `MiniExecutor`.
static VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_waker,
    wake_waker,
    wake_by_ref_waker,
    drop_waker,
);

// This function is responsible for cloning the waker. 
// It takes a raw pointer (ptr) to the MiniExecutor, reconstructs the Arc<MiniExecutor> from the raw pointer, c
// reates a new RawWaker by cloning the Arc, and then forgets the original Arc to avoid double-dropping. 
// This function is called when a waker is cloned.
unsafe fn clone_waker(ptr: *const ()) -> RawWaker {
    let executor = Arc::from_raw(ptr.cast::<MiniExecutor>());
    let raw_waker = RawWaker::new(Arc::into_raw(executor.clone()).cast::<()>(), &VTABLE);
    std::mem::forget(executor);
    raw_waker
}

// This function is responsible for waking the waker. 
// It takes a raw pointer (ptr) to the MiniExecutor, reconstructs the Arc<MiniExecutor> from the raw pointer, and runs the executor. 
// This function is called when a waker is woken up and needs to be executed.
unsafe fn wake_waker(ptr: *const ()) {
    let executor = Arc::from_raw(ptr.cast::<MiniExecutor>());
    executor.run();
}

// This function is responsible for waking the waker by reference. 
// However, since this is a single-threaded executor, we don't need to do anything here. 
// In a multi-threaded executor, you might need to notify the executor to resume executing the associated future.
unsafe fn wake_by_ref_waker(_ptr: *const ()) {
    // Do nothing, as this is a single-threaded executor.
}

// This function is responsible for dropping the waker. 
// It takes a raw pointer (ptr) to the MiniExecutor, reconstructs the Arc<MiniExecutor> from the raw pointer, and then drops it. 
// This function is called when a waker is dropped and its resources need to be released.
unsafe fn drop_waker(ptr: *const ()) {
    drop(Arc::from_raw(ptr.cast::<MiniExecutor>()));
}
