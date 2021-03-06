//! The Tokio runtime.
//!
//! Unlike other Rust programs, asynchronous applications require
//! runtime support. In particular, the following runtime services are
//! necessary:
//!
//! * An **I/O event loop**, called the driver, which drives I/O resources and
//!   dispatches I/O events to tasks that depend on them.
//! * A **scheduler** to execute [tasks] that use these I/O resources.
//! * A **timer** for scheduling work to run after a set period of time.
//!
//! Tokio's [`Runtime`] bundles all of these services as a single type, allowing
//! them to be started, shut down, and configured together. However, often
//! it is not required to configure a [`Runtime`] manually, and user may just
//! use the [`tokio::main`] attribute macro, which creates a [`Runtime`] under
//! the hood.
//!
//! # Usage
//!
//! When no fine tuning is required, the [`tokio::main`] attribute macro can be used.
//!
//! ```no_run
//! use tokio::net::TcpListener;
//! use tokio::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!     loop {
//!         let (mut socket, _) = listener.accept().await?;
//!
//!         tokio::spawn(async move {
//!             let mut buf = [0; 1024];
//!
//!             // In a loop, read data from the socket and write the data back.
//!             loop {
//!                 let n = match socket.read(&mut buf).await {
//!                     // socket closed
//!                     Ok(n) if n == 0 => return,
//!                     Ok(n) => n,
//!                     Err(e) => {
//!                         println!("failed to read from socket; err = {:?}", e);
//!                         return;
//!                     }
//!                 };
//!
//!                 // Write the data back
//!                 if let Err(e) = socket.write_all(&buf[0..n]).await {
//!                     println!("failed to write to socket; err = {:?}", e);
//!                     return;
//!                 }
//!             }
//!         });
//!     }
//! }
//! ```
//!
//! From within the context of the runtime, additional tasks are spawned using
//! the [`tokio::spawn`] function. Futures spawned using this function will be
//! executed on the same thread pool used by the [`Runtime`].
//!
//! A [`Runtime`] instance can also be used directly.
//!
//! ```no_run
//! use tokio::net::TcpListener;
//! use tokio::prelude::*;
//! use tokio::runtime::Runtime;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create the runtime
//!     let rt  = Runtime::new()?;
//!
//!     // Spawn the root task
//!     rt.block_on(async {
//!         let mut listener = TcpListener::bind("127.0.0.1:8080").await?;
//!
//!         loop {
//!             let (mut socket, _) = listener.accept().await?;
//!
//!             tokio::spawn(async move {
//!                 let mut buf = [0; 1024];
//!
//!                 // In a loop, read data from the socket and write the data back.
//!                 loop {
//!                     let n = match socket.read(&mut buf).await {
//!                         // socket closed
//!                         Ok(n) if n == 0 => return,
//!                         Ok(n) => n,
//!                         Err(e) => {
//!                             println!("failed to read from socket; err = {:?}", e);
//!                             return;
//!                         }
//!                     };
//!
//!                     // Write the data back
//!                     if let Err(e) = socket.write_all(&buf[0..n]).await {
//!                         println!("failed to write to socket; err = {:?}", e);
//!                         return;
//!                     }
//!                 }
//!             });
//!         }
//!     })
//! }
//! ```
//!
//! ## Runtime Configurations
//!
//! Tokio provides multiple task scheduling strategies, suitable for different
//! applications. The [runtime builder] or `#[tokio::main]` attribute may be
//! used to select which scheduler to use.
//!
//! #### Basic Scheduler
//!
//! The basic scheduler provides a _single-threaded_ future executor. All tasks
//! will be created and executed on the current thread. The basic scheduler
//! requires the `rt-core` feature flag, and can be selected using the
//! [`Builder::basic_scheduler`] method:
//! ```
//! use tokio::runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let basic_rt = runtime::Builder::new()
//!     .basic_scheduler()
//!     .build()?;
//! # Ok(()) }
//! ```
//!
//! If the `rt-core` feature is enabled and `rt-threaded` is not,
//! [`Runtime::new`] will return a basic scheduler runtime by default.
//!
//! #### Threaded Scheduler
//!
//! The threaded scheduler executes futures on a _thread pool_, using a
//! work-stealing strategy. By default, it will start a worker thread for each
//! CPU core available on the system. This tends to be the ideal configurations
//! for most applications. The threaded scheduler requires the `rt-threaded` feature
//! flag, and can be selected using the  [`Builder::threaded_scheduler`] method:
//! ```
//! use tokio::runtime;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let threaded_rt = runtime::Builder::new()
//!     .threaded_scheduler()
//!     .build()?;
//! # Ok(()) }
//! ```
//!
//! If the `rt-threaded` feature flag is enabled, [`Runtime::new`] will return a
//! threaded scheduler runtime by default.
//!
//! Most applications should use the threaded scheduler, except in some niche
//! use-cases, such as when running only a single thread is required.
//!
//! #### Resource drivers
//!
//! When configuring a runtime by hand, no resource drivers are enabled by
//! default. In this case, attempting to use networking types or time types will
//! fail. In order to enable these types, the resource drivers must be enabled.
//! This is done with [`Builder::enable_io`] and [`Builder::enable_time`]. As a
//! shorthand, [`Builder::enable_all`] enables both resource drivers.
//!
//! ## Lifetime of spawned threads
//!
//! The runtime may spawn threads depending on its configuration and usage. The
//! threaded scheduler spawns threads to schedule tasks and calls to
//! `spawn_blocking` spawn threads to run blocking operations.
//!
//! While the `Runtime` is active, threads may shutdown after periods of being
//! idle. Once `Runtime` is dropped, all runtime threads are forcibly shutdown.
//! Any tasks that have not yet completed will be dropped.
//!
//! [tasks]: crate::task
//! [`Runtime`]: Runtime
//! [`tokio::spawn`]: crate::spawn
//! [`tokio::main`]: ../attr.main.html
//! [runtime builder]: crate::runtime::Builder
//! [`Runtime::new`]: crate::runtime::Runtime::new
//! [`Builder::basic_scheduler`]: crate::runtime::Builder::basic_scheduler
//! [`Builder::threaded_scheduler`]: crate::runtime::Builder::threaded_scheduler
//! [`Builder::enable_io`]: crate::runtime::Builder::enable_io
//! [`Builder::enable_time`]: crate::runtime::Builder::enable_time
//! [`Builder::enable_all`]: crate::runtime::Builder::enable_all

// At the top due to macros
#[cfg(test)]
#[macro_use]
mod tests;

pub(crate) mod context;

cfg_rt_core! {
    mod basic_scheduler;
    use basic_scheduler::BasicScheduler;

    pub(crate) mod task;
}

mod blocking;
use blocking::BlockingPool;

cfg_blocking_impl! {
    #[allow(unused_imports)]
    pub(crate) use blocking::{spawn_blocking, try_spawn_blocking};
}

mod builder;
pub use self::builder::Builder;

pub(crate) mod driver;

pub(crate) mod enter;
use self::enter::enter;

mod handle;
use handle::Handle;

mod io {
    /// Re-exported for convenience.
    pub(crate) use std::io::Result;
}

cfg_rt_threaded! {
    mod park;
    use park::Parker;
}

mod shell;
use self::shell::Shell;

mod spawner;
use self::spawner::Spawner;

cfg_rt_threaded! {
    mod queue;

    pub(crate) mod thread_pool;
    use self::thread_pool::ThreadPool;
}

cfg_rt_core! {
    use crate::task::JoinHandle;
}

use crate::loom::sync::Mutex;
use std::future::Future;
use std::time::Duration;

/// The Tokio runtime.
///
/// The runtime provides an I/O driver, task scheduler, [timer], and blocking
/// pool, necessary for running asynchronous tasks.
///
/// Instances of `Runtime` can be created using [`new`] or [`Builder`]. However,
/// most users will use the `#[tokio::main]` annotation on their entry point instead.
///
/// See [module level][mod] documentation for more details.
///
/// # Shutdown
///
/// Shutting down the runtime is done by dropping the value. The current thread
/// will block until the shut down operation has completed.
///
/// * Drain any scheduled work queues.
/// * Drop any futures that have not yet completed.
/// * Drop the reactor.
///
/// Once the reactor has dropped, any outstanding I/O resources bound to
/// that reactor will no longer function. Calling any method on them will
/// result in an error.
///
/// [timer]: crate::time
/// [mod]: index.html
/// [`new`]: method@Self::new
/// [`Builder`]: struct@Builder
/// [`tokio::run`]: fn@run
#[derive(Debug)]
pub struct Runtime {
    /// Task executor
    kind: Kind,

    /// Handle to runtime, also contains driver handles
    handle: Handle,

    /// Blocking pool handle, used to signal shutdown
    blocking_pool: BlockingPool,
}

/// The runtime executor is either a thread-pool or a current-thread executor.
#[derive(Debug)]
enum Kind {
    /// Not able to execute concurrent tasks. This variant is mostly used to get
    /// access to the driver handles.
    Shell(Mutex<Option<Shell>>),

    /// Execute all tasks on the current-thread.
    #[cfg(feature = "rt-core")]
    Basic(Mutex<Option<BasicScheduler<driver::Driver>>>),

    /// Execute tasks across multiple threads.
    #[cfg(feature = "rt-threaded")]
    ThreadPool(ThreadPool),
}

/// After thread starts / before thread stops
type Callback = std::sync::Arc<dyn Fn() + Send + Sync>;

impl Runtime {
    /// Create a new runtime instance with default configuration values.
    ///
    /// This results in a scheduler, I/O driver, and time driver being
    /// initialized. The type of scheduler used depends on what feature flags
    /// are enabled: if the `rt-threaded` feature is enabled, the [threaded
    /// scheduler] is used, while if only the `rt-core` feature is enabled, the
    /// [basic scheduler] is used instead.
    ///
    /// If the threaded scheduler is selected, it will not spawn
    /// any worker threads until it needs to, i.e. tasks are scheduled to run.
    ///
    /// Most applications will not need to call this function directly. Instead,
    /// they will use the  [`#[tokio::main]` attribute][main]. When more complex
    /// configuration is necessary, the [runtime builder] may be used.
    ///
    /// See [module level][mod] documentation for more details.
    ///
    /// # Examples
    ///
    /// Creating a new `Runtime` with default configuration values.
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new()
    ///     .unwrap();
    ///
    /// // Use the runtime...
    /// ```
    ///
    /// [mod]: index.html
    /// [main]: ../attr.main.html
    /// [threaded scheduler]: index.html#threaded-scheduler
    /// [basic scheduler]: index.html#basic-scheduler
    /// [runtime builder]: crate::runtime::Builder
    pub fn new() -> io::Result<Runtime> {
        #[cfg(feature = "rt-threaded")]
        let ret = Builder::new().threaded_scheduler().enable_all().build();

        #[cfg(all(not(feature = "rt-threaded"), feature = "rt-core"))]
        let ret = Builder::new().basic_scheduler().enable_all().build();

        #[cfg(not(feature = "rt-core"))]
        let ret = Builder::new().enable_all().build();

        ret
    }

    /// Spawn a future onto the Tokio runtime.
    ///
    /// This spawns the given future onto the runtime's executor, usually a
    /// thread pool. The thread pool is then responsible for polling the future
    /// until it completes.
    ///
    /// See [module level][mod] documentation for more details.
    ///
    /// [mod]: index.html
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// # fn dox() {
    /// // Create the runtime
    /// let rt = Runtime::new().unwrap();
    ///
    /// // Spawn a future onto the runtime
    /// rt.spawn(async {
    ///     println!("now running on a worker thread");
    /// });
    /// # }
    /// ```
    ///
    /// # Panics
    ///
    /// This function will not panic unless task execution is disabled on the
    /// executor. This can only happen if the runtime was built using
    /// [`Builder`] without picking either [`basic_scheduler`] or
    /// [`threaded_scheduler`].
    ///
    /// [`Builder`]: struct@Builder
    /// [`threaded_scheduler`]: fn@Builder::threaded_scheduler
    /// [`basic_scheduler`]: fn@Builder::basic_scheduler
    #[cfg(feature = "rt-core")]
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        match &self.kind {
            Kind::Shell(_) => panic!("task execution disabled"),
            #[cfg(feature = "rt-threaded")]
            Kind::ThreadPool(exec) => exec.spawn(future),
            Kind::Basic(_exec) => self.handle.spawner.spawn(future),
        }
    }

    /// Run a future to completion on the Tokio runtime. This is the runtime's
    /// entry point.
    ///
    /// This runs the given future on the runtime, blocking until it is
    /// complete, and yielding its resolved result. Any tasks or timers which
    /// the future spawns internally will be executed on the runtime.
    ///
    /// When this runtime is configured with `core_threads = 0`, only the first call
    /// to `block_on` will run the IO and timer drivers. Calls to other methods _before_ the first
    /// `block_on` completes will just hook into the driver running on the thread
    /// that first called `block_on`. This means that the driver may be passed
    /// from thread to thread by the user between calls to `block_on`.
    ///
    /// This method may not be called from an asynchronous context.
    ///
    /// # Panics
    ///
    /// This function panics if the provided future panics, or if called within an
    /// asynchronous execution context.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tokio::runtime::Runtime;
    ///
    /// // Create the runtime
    /// let rt  = Runtime::new().unwrap();
    ///
    /// // Execute the future, blocking the current thread until completion
    /// rt.block_on(async {
    ///     println!("hello");
    /// });
    /// ```
    ///
    /// [handle]: fn@Handle::block_on
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.handle.enter(|| match &self.kind {
            Kind::Shell(exec) => {
                // TODO(lucio): clean this up and move this impl into
                // `shell.rs`, this is hacky and bad but will work for
                // now.
                let exec_temp = {
                    let mut lock = exec.lock().unwrap();
                    lock.take()
                };

                if let Some(mut exec_temp) = exec_temp {
                    let res = exec_temp.block_on(future);
                    exec.lock().unwrap().replace(exec_temp);
                    res
                } else {
                    let mut enter = crate::runtime::enter(true);
                    enter.block_on(future).unwrap()
                }
            }
            #[cfg(feature = "rt-core")]
            Kind::Basic(exec) => {
                // TODO(lucio): clean this up and move this impl into
                // `basic_scheduler.rs`, this is hacky and bad but will work for
                // now.
                let exec_temp = {
                    let mut lock = exec.lock().unwrap();
                    lock.take()
                };

                if let Some(mut exec_temp) = exec_temp {
                    let res = exec_temp.block_on(future);
                    exec.lock().unwrap().replace(exec_temp);
                    res
                } else {
                    let mut enter = crate::runtime::enter(true);
                    enter.block_on(future).unwrap()
                }
            }
            #[cfg(feature = "rt-threaded")]
            Kind::ThreadPool(exec) => exec.block_on(future),
        })
    }

    /// Enter the runtime context. This allows you to construct types that must
    /// have an executor available on creation such as [`Delay`] or [`TcpStream`].
    /// It will also allow you to call methods such as [`tokio::spawn`].
    ///
    /// [`Delay`]: struct@crate::time::Delay
    /// [`TcpStream`]: struct@crate::net::TcpStream
    /// [`tokio::spawn`]: fn@crate::spawn
    ///
    /// # Example
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// fn function_that_spawns(msg: String) {
    ///     // Had we not used `rt.enter` below, this would panic.
    ///     tokio::spawn(async move {
    ///         println!("{}", msg);
    ///     });
    /// }
    ///
    /// fn main() {
    ///     let rt = Runtime::new().unwrap();
    ///
    ///     let s = "Hello World!".to_string();
    ///
    ///     // By entering the context, we tie `tokio::spawn` to this executor.
    ///     rt.enter(|| function_that_spawns(s));
    /// }
    /// ```
    pub fn enter<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        self.handle.enter(f)
    }

    /// Shutdown the runtime, waiting for at most `duration` for all spawned
    /// task to shutdown.
    ///
    /// Usually, dropping a `Runtime` handle is sufficient as tasks are able to
    /// shutdown in a timely fashion. However, dropping a `Runtime` will wait
    /// indefinitely for all tasks to terminate, and there are cases where a long
    /// blocking task has been spawned, which can block dropping `Runtime`.
    ///
    /// In this case, calling `shutdown_timeout` with an explicit wait timeout
    /// can work. The `shutdown_timeout` will signal all tasks to shutdown and
    /// will wait for at most `duration` for all spawned tasks to terminate. If
    /// `timeout` elapses before all tasks are dropped, the function returns and
    /// outstanding tasks are potentially leaked.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    /// use tokio::task;
    ///
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// fn main() {
    ///    let runtime = Runtime::new().unwrap();
    ///
    ///    runtime.block_on(async move {
    ///        task::spawn_blocking(move || {
    ///            thread::sleep(Duration::from_secs(10_000));
    ///        });
    ///    });
    ///
    ///    runtime.shutdown_timeout(Duration::from_millis(100));
    /// }
    /// ```
    pub fn shutdown_timeout(mut self, duration: Duration) {
        // Wakeup and shutdown all the worker threads
        self.handle.spawner.shutdown();
        self.blocking_pool.shutdown(Some(duration));
    }

    /// Shutdown the runtime, without waiting for any spawned tasks to shutdown.
    ///
    /// This can be useful if you want to drop a runtime from within another runtime.
    /// Normally, dropping a runtime will block indefinitely for spawned blocking tasks
    /// to complete, which would normally not be permitted within an asynchronous context.
    /// By calling `shutdown_background()`, you can drop the runtime from such a context.
    ///
    /// Note however, that because we do not wait for any blocking tasks to complete, this
    /// may result in a resource leak (in that any blocking tasks are still running until they
    /// return.
    ///
    /// This function is equivalent to calling `shutdown_timeout(Duration::of_nanos(0))`.
    ///
    /// ```
    /// use tokio::runtime::Runtime;
    ///
    /// fn main() {
    ///    let runtime = Runtime::new().unwrap();
    ///
    ///    runtime.block_on(async move {
    ///        let inner_runtime = Runtime::new().unwrap();
    ///        // ...
    ///        inner_runtime.shutdown_background();
    ///    });
    /// }
    /// ```
    pub fn shutdown_background(self) {
        self.shutdown_timeout(Duration::from_nanos(0))
    }
}
