use args::{
  ArgsError,
  Argument,
  ArgumentParser,
  ArgumentType,
};

fn main() {
  // --json and --yaml are mutually exclusive; --out requires --format
  let parser = ArgumentParser::new("exclusive")
    .with_exclusive_group(&["--json", "--yaml"])
    .with_requires("--out", "--format")
    .with_argument(Argument::new("--json").with_type(ArgumentType::Boolean))
    .with_argument(Argument::new("--yaml").with_type(ArgumentType::Boolean))
    .with_argument(Argument::new("--format").with_type(ArgumentType::String))
    .with_argument(Argument::new("--out").with_type(ArgumentType::String));

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      let fmt = parsed
        .get_string("--format")
        .unwrap_or_else(|| "json".into());
      let out = parsed
        .get_string("--out")
        .unwrap_or_else(|| "stdout".into());
      println!("fmt={}, out={}", fmt, out);
    }
    Err(ArgsError::InvalidValue { name, expected, .. }) => {
      eprintln!("Validation error on {}: {}", name, expected);
    }
    Err(e) => eprintln!("{}", e),
  }
}
