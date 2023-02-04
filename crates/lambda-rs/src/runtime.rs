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
  fn before_start(&mut self);
  fn run(self) -> Result<RuntimeResult, RuntimeError>;
}

/// Starts a runtime and waits for it to finish. This function will not return
/// until the runtime has finished executing.
///
/// The type `ImplementedRuntime` represents any struct which implements the
/// `Runtime` trait with valid `RuntimeResult` & `RuntimeError` parameters.
pub fn start_runtime<
  RuntimeResult: Sized + Debug,
  RuntimeError: Sized + Debug,
  ImplementedRuntime: Runtime<RuntimeResult, RuntimeError>,
>(
  runtime: ImplementedRuntime,
) {
  let runtime_result = runtime.run();
  match runtime_result {
    Ok(result) => {
      logging::info!(
        "Runtime finished successfully with the result: {:?}",
        result
      );
    }
    Err(e) => {
      logging::fatal!("Runtime panicked because: {:?}", e);
    }
  }
}
