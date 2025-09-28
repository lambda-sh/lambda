use args::{Argument, ArgumentParser, ArgumentType, ArgumentValue};

fn main() {
  let parser = ArgumentParser::new("basic")
    .with_description("Basic parse() example with required/optional args")
    .with_author("lambda team")
    .with_argument(
      Argument::new("--name")
        .is_required(true)
        .with_type(ArgumentType::String),
    )
    .with_argument(
      Argument::new("--count")
        .with_type(ArgumentType::Integer)
        .with_default_value(ArgumentValue::Integer(1)),
    );

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      let name = parsed.get_string("--name").unwrap();
      let count = parsed.get_i64("--count").unwrap();
      println!("name={}, count={}", name, count);
    }
    Err(e) => {
      // HelpRequested prints usage text via Display
      eprintln!("{}", e);
    }
  }
}
