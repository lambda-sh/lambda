# lambda-rs-args
![lambda-rs](https://img.shields.io/crates/d/lambda-rs-args)
![lambda-rs](https://img.shields.io/crates/v/lambda-rs-args)

Lightweight, dependency-free CLI argument parsing used by lambda tools.

It provides a simple builder API, a non-panicking `parse` that returns rich errors, auto-generated usage/help, and pragmatic features like booleans with `--no-flag`, short-flag clusters and counts, positionals, env/config merging, subcommands, and validation helpers.

See the examples in `crates/lambda-rs-args/examples/` for runnable snippets.

## Quick Start

Code:

```rust
use args::{Argument, ArgumentParser, ArgumentType, ArgumentValue};

fn main() {
  let parser = ArgumentParser::new("my-tool")
    .with_description("Demo tool")
    .with_author("you")
    .with_argument(Argument::new("--name").is_required(true).with_type(ArgumentType::String))
    .with_argument(Argument::new("--count").with_type(ArgumentType::Integer).with_default_value(ArgumentValue::Integer(1)));

  let argv: Vec<String> = std::env::args().collect();
  match parser.parse(&argv) {
    Ok(parsed) => {
      println!("name={}, count={}", parsed.get_string("--name").unwrap(), parsed.get_i64("--count").unwrap());
    }
    Err(e) => eprintln!("{}", e), // includes --help output
  }
}
```

CLI:

```
$ my-tool --help
Usage: my-tool [options]

Demo tool

Author(s): you

Options:
  --name <string>
       (required)
  --count <int>
       [default: 1]
$ my-tool --name Alice
name=Alice, count=1
```

## Features and Examples

Each feature below shows a minimal code snippet and the CLI it enables.

- Non‑panicking parse with help
  - Code: see Quick Start (match on `Err(ArgsError::HelpRequested(usage))`).
  - CLI: `--help`, `-h` prints usage and exits with error containing the usage text.

- Booleans with presence and `--no-flag`
  - Code (examples/bools.rs):
    ```rust
    let parser = ArgumentParser::new("bools")
      .with_argument(Argument::new("--verbose").with_type(ArgumentType::Boolean))
      .with_argument(Argument::new("--dry-run").with_type(ArgumentType::Boolean));
    ```
  - CLI:
    ```
    $ bools --verbose --no-dry-run
    verbose=true, dry_run=false
    ```

- `--arg=value` equals syntax
  - Code (examples/equals.rs):
    ```rust
    let parser = ArgumentParser::new("equals")
      .with_argument(Argument::new("--threshold").with_type(ArgumentType::Float))
      .with_argument(Argument::new("--title").with_type(ArgumentType::String));
    ```
  - CLI:
    ```
    $ equals --threshold=0.75 --title=Demo
    threshold=0.75, title=Demo
    ```

- Short flags, clusters, and counting verbosity
  - Code (examples/short_count.rs):
    ```rust
    let parser = ArgumentParser::new("short-count")
      .with_argument(Argument::new("-v").with_aliases(&["-v"]).with_type(ArgumentType::Count));
    ```
  - CLI:
    ```
    $ short-count -vvv
    verbosity=3
    $ short-count -v -v
    verbosity=2
    ```

- Aliases (short and long)
  - Code:
    ```rust
    Argument::new("--output").with_type(ArgumentType::String).with_aliases(&["-o"])
    ```
  - CLI:
    ```
    $ tool -o out.bin
    ```

- Positional arguments and `--` terminator
  - Code (examples/positionals.rs):
    ```rust
    let parser = ArgumentParser::new("pos")
      .with_argument(Argument::new("input").as_positional().with_type(ArgumentType::String))
      .with_argument(Argument::new("output").as_positional().with_type(ArgumentType::String));
    ```
  - CLI:
    ```
    $ pos -- fileA fileB
    fileA -> fileB
    ```

- Subcommands
  - Code (examples/subcommands.rs):
    ```rust
    let root = ArgumentParser::new("tool")
      .with_subcommand(ArgumentParser::new("serve").with_argument(Argument::new("--port").with_type(ArgumentType::Integer)))
      .with_subcommand(ArgumentParser::new("build").with_argument(Argument::new("--release").with_type(ArgumentType::Boolean)));
    // parsed.subcommand() -> Option<(name, &ParsedArgs)>
    ```
  - CLI:
    ```
    $ tool serve --port 8080
    serving on :8080
    $ tool build --release
    building (release=true)
    ```

- Env var merge (prefix) and simple config file
  - Code (examples/env_config.rs):
    ```rust
    let parser = ArgumentParser::new("envcfg")
      .with_env_prefix("APP")
      .with_config_file("app.cfg")
      .with_argument(Argument::new("--host").with_type(ArgumentType::String))
      .with_argument(Argument::new("--port").with_type(ArgumentType::Integer));
    ```
  - Env and config format:
    - Env: `APP_HOST=127.0.0.1`, `APP_PORT=8080`
    - Config: `app.cfg` lines like `--host=127.0.0.1` or `HOST=127.0.0.1`
  - CLI:
    ```
    $ APP_PORT=5000 envcfg
    0.0.0.0:5000
    ```

- Validation: requires and mutually exclusive
  - Code (examples/exclusives.rs):
    ```rust
    let parser = ArgumentParser::new("exclusive")
      .with_exclusive_group(&["--json", "--yaml"]).with_requires("--out", "--format")
      .with_argument(Argument::new("--json").with_type(ArgumentType::Boolean))
      .with_argument(Argument::new("--yaml").with_type(ArgumentType::Boolean))
      .with_argument(Argument::new("--format").with_type(ArgumentType::String))
      .with_argument(Argument::new("--out").with_type(ArgumentType::String));
    ```
  - CLI:
    ```
    $ exclusive --json --yaml
    Validation error on --json, --yaml: mutually exclusive (choose one)
    $ exclusive --out out.json
    Validation error on --out: requires --format
    ```

- Ignore unknown flags
  - Code:
    ```rust
    let parser = ArgumentParser::new("tool").ignore_unknown(true);
    ```
  - CLI: `tool --unknown --still-works`

## More Useful Features (Explained)

- Booleans and `--no-flag`
  - Presence sets to true (e.g., `--verbose`), and `--no-verbose` sets to false.
  - Works well for toggles and is more ergonomic than `--verbose=false`.

- Short clusters and counting
  - `-vvv` increments a `Count` argument three times; useful for verbosity.
  - Also works with separated flags: `-v -v`.

- Env + Config merging
  - Handy for defaulting from environment or a checked-in config file.
  - CLI always wins over env, and env wins over config.
  - Uses a simple `key=value` format; both canonical names (`--host`) and uppercase keys (`HOST`) are supported.

- Subcommands
  - Let you structure CLIs (e.g., `tool serve`, `tool build`).
  - Each subcommand has its own parser and arguments. Use `parsed.subcommand()` to route.

- Validation helpers
  - `.with_requires(a, b)` enforces that if `a` is provided, `b` must be too.
  - `.with_exclusive_group([a, b, c])` ensures only one of them is present.

## Design Notes

- Non-panicking `parse` returns `Result<ParsedArgs, ArgsError>` for predictable flows and better UX.
- `compile` remains for backward compatibility and will panic on error with friendly messages.
- Dependency-free; keeps binary sizes small and build times fast.

## Examples

- See all examples under `crates/lambda-rs-args/examples/`:
  - `basic.rs`, `bools.rs`, `equals.rs`, `short_count.rs`, `positionals.rs`, `subcommands.rs`, `env_config.rs`, `exclusives.rs`.
