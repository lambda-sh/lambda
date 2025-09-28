use args::{Argument, ArgumentParser, ArgumentType};

fn main() {
  // Reads APP_HOST and APP_PORT if set. Also reads from ./app.cfg if present
  // with lines like: --host=127.0.0.1
  let parser = ArgumentParser::new("envcfg")
    .with_env_prefix("APP")
    .with_config_file("app.cfg")
    .with_argument(Argument::new("--host").with_type(ArgumentType::String))
    .with_argument(Argument::new("--port").with_type(ArgumentType::Integer));

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      let host = parsed
        .get_string("--host")
        .unwrap_or_else(|| "0.0.0.0".into());
      let port = parsed.get_i64("--port").unwrap_or(3000);
      println!("{}:{}", host, port);
    }
    Err(e) => eprintln!("{}", e),
  }
}
