use args::{Argument, ArgumentParser, ArgumentType};

fn main() {
  // root
  let root =
    ArgumentParser::new("tool")
      .with_description("Demo with subcommands: serve/build")
      .with_subcommand(ArgumentParser::new("serve").with_argument(
        Argument::new("--port").with_type(ArgumentType::Integer),
      ))
      .with_subcommand(ArgumentParser::new("build").with_argument(
        Argument::new("--release").with_type(ArgumentType::Boolean),
      ));

  let args: Vec<String> = std::env::args().collect();
  let usage = root.usage();
  match root.parse(&args) {
    Ok(parsed) => {
      if let Some((name, sub)) = parsed.subcommand() {
        match name {
          "serve" => {
            let port = sub.get_i64("--port").unwrap_or(8080);
            println!("serving on :{}", port);
          }
          "build" => {
            let rel = sub.get_bool("--release").unwrap_or(false);
            println!("building (release={})", rel);
          }
          _ => {}
        }
      } else {
        eprintln!("No subcommand provided.\n{}", usage);
      }
    }
    Err(e) => eprintln!("{}", e),
  }
}
