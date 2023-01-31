//! Runtime definition & functions for executing lambda applications.

use std::fmt::Debug;

use logging;

/// A runtime is an important but simple type in lambda that is responsible for
/// executing the application. The event loop for the application is started
/// within the runtime and should live for the duration of the application.
pub trait Runtime<RuntimeResult, RuntimeError>
where
  RuntimeResult: Sized + Debug,
  RuntimeError: Sized + Debug,
{
  type Component;
  fn on_start(&mut self);
  fn on_stop(&mut self);
  fn run(self) -> Result<RuntimeResult, RuntimeError>;
}

/// Simple function for starting any prebuilt Runnable.
pub fn start_runtime<R: Sized + Debug, E: Sized + Debug, T: Runtime<R, E>>(
  runtime: T,
) {
  let runtime_result = runtime.run();
  match runtime_result {
    Ok(_) => {
      logging::info!("Runtime finished successfully.");
    }
    Err(e) => {
      logging::fatal!("Runtime panicked because: {:?}", e);
    }
  }
}
