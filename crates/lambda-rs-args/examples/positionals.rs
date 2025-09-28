use args::{Argument, ArgumentParser, ArgumentType};

fn main() {
  let parser = ArgumentParser::new("pos")
    .with_argument(
      Argument::new("input")
        .as_positional()
        .with_type(ArgumentType::String),
    )
    .with_argument(
      Argument::new("output")
        .as_positional()
        .with_type(ArgumentType::String),
    );

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      println!(
        "{} -> {}",
        parsed.get_string("input").unwrap_or_default(),
        parsed.get_string("output").unwrap_or_default()
      );
    }
    Err(e) => eprintln!("{}", e),
  }
}
