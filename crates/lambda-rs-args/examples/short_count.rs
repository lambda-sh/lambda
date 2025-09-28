use args::{Argument, ArgumentParser, ArgumentType};

fn main() {
  let parser = ArgumentParser::new("short-count").with_argument(
    Argument::new("-v")
      .with_aliases(&["-v"])
      .with_type(ArgumentType::Count),
  );

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      println!("verbosity={}", parsed.get_count("-v").unwrap_or(0));
    }
    Err(e) => eprintln!("{}", e),
  }
}
