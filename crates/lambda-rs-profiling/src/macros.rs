/// Begins a profiling scope and returns a bindable guard value.
#[macro_export]
macro_rules! profile_scope {
  ($name:expr) => {{
    let lambda_profile_scope_name: &'static str = $name;

    $crate::__private::begin_scope(lambda_profile_scope_name)
  }};
}

/// Begins a profiling scope using a stable fallback name for the current
/// function.
#[macro_export]
macro_rules! profile_function {
  () => {{
    $crate::profile_scope!($crate::__private::function_scope_name(
      ::core::module_path!()
    ))
  }};
}

/// Records a profiling counter when a global profiler is installed.
#[macro_export]
macro_rules! profile_counter {
  ($name:expr, $value:expr) => {{
    let lambda_profile_counter_name: &'static str = $name;
    let lambda_profile_counter_value: u64 = $value as u64;

    $crate::__private::record_counter(
      lambda_profile_counter_name,
      lambda_profile_counter_value,
    );
  }};
}

/// Records a profiling instant when a global profiler is installed.
#[macro_export]
macro_rules! profile_instant {
  ($name:expr) => {{
    let lambda_profile_instant_name: &'static str = $name;

    $crate::__private::record_instant(lambda_profile_instant_name);
  }};
}
