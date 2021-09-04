trait Logger {
  pub fn new(name: &str) -> Self;
  pub fn info();
  pub fn error();
  pub fn warn();
  pub fn fatal();
}

pub struct LambdaLogger {
  name: String,
  call_count: u32,
}

impl Logger for LambdaLogger {
  fn new(name: &str) -> Self {
    return LambdaLogger{
      name,
      call_count: 0
    };
  }

  fn info() {
    
  }
}