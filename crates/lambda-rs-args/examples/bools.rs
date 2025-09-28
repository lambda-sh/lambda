use args::{
  Argument,
  ArgumentParser,
  ArgumentType,
};

fn main() {
  let parser = ArgumentParser::new("bools")
    .with_description("Boolean flags: presence sets true; --no-flag sets false")
    .with_argument(Argument::new("--verbose").with_type(ArgumentType::Boolean))
    .with_argument(Argument::new("--dry-run").with_type(ArgumentType::Boolean));

  let args: Vec<String> = std::env::args().collect();
  match parser.parse(&args) {
    Ok(parsed) => {
      let verbose = parsed.get_bool("--verbose").unwrap_or(false);
      let dry_run = parsed.get_bool("--dry-run").unwrap_or(false);
      println!("verbose={}, dry_run={}", verbose, dry_run);
    }
    Err(e) => {
      eprintln!("{}", e);
    }
  }
}
