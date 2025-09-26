use args::{
  Argument,
  ArgumentParser,
  ArgumentType,
};

fn main() {
  let parser = ArgumentParser::new("equals")
    .with_description("--arg=value style parsing")
    .with_argument(Argument::new("--threshold").with_type(ArgumentType::Float))
    .with_argument(Argument::new("--title").with_type(ArgumentType::String));

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      let threshold = parsed.get_f32("--threshold").unwrap_or(0.5);
      let title = parsed
        .get_string("--title")
        .unwrap_or_else(|| "(none)".to_string());
      println!("threshold={}, title={}", threshold, title);
    }
    Err(e) => {
      eprintln!("{}", e);
    }
  }
}
