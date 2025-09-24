//! # Lambda Args
//! Lambda Args is a simple argument parser for Rust. It is designed to be
//! simple to use and primarily for use in lambda command line applications.

use std::collections::HashMap;
use std::fmt;

pub struct ArgumentParser {
  name: String,
  description: String,
  authors: Vec<String>,
  args: HashMap<String, (Argument, bool, usize)>,
  aliases: HashMap<String, String>,
  positionals: Vec<String>,
  env_prefix: Option<String>,
  ignore_unknown: bool,
  exclusive_groups: Vec<Vec<String>>,
  requires: Vec<(String, String)>,
  config_path: Option<String>,
  subcommands: HashMap<String, ArgumentParser>,
  is_subcommand: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ArgumentType {
  Boolean,
  Integer,
  Float,
  Double,
  String,
  Count,
  StringList,
  IntegerList,
  FloatList,
  DoubleList,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ArgumentValue {
  None,
  Boolean(bool),
  Integer(i64),
  Float(f32),
  Double(f64),
  String(String),
}

impl Into<String> for ArgumentValue {
  fn into(self) -> String {
    return match self {
      ArgumentValue::String(val) => val,
      _ => panic!("Cannot convert {:?} into a String.", self),
    };
  }
}

impl Into<i64> for ArgumentValue {
  fn into(self) -> i64 {
    return match self {
      ArgumentValue::Integer(val) => val,
      ArgumentValue::Float(val) => val as i64,
      ArgumentValue::Double(val) => val as i64,
      _ => panic!("Cannot convert {:?} into an i64", self),
    };
  }
}

impl Into<f32> for ArgumentValue {
  fn into(self) -> f32 {
    return match self {
      ArgumentValue::Float(val) => val,
      _ => panic!("Cannot convert {:?} into a f32", self),
    };
  }
}

impl Into<f64> for ArgumentValue {
  fn into(self) -> f64 {
    return match self {
      ArgumentValue::Double(val) => val,
      ArgumentValue::Float(val) => val as f64,
      ArgumentValue::Integer(val) => val as f64,
      _ => panic!("Cannot convert {:?} into a f64", self),
    };
  }
}

#[derive(Debug)]
pub struct Argument {
  name: String,
  description: String,
  required: bool,
  arg_type: ArgumentType,
  default_value: ArgumentValue,
  aliases: Vec<String>,
  positional: bool,
}

impl Argument {
  /// Creates a new argument where the name represents
  pub fn new(name: &str) -> Self {
    return Argument {
      name: name.to_string(),
      description: String::new(),
      required: false,
      arg_type: ArgumentType::String,
      default_value: ArgumentValue::None,
      aliases: vec![],
      positional: false,
    };
  }

  /// Sets the Argument explicitly as required or not.
  pub fn is_required(mut self, required: bool) -> Self {
    self.required = required;
    return self;
  }

  /// Sets the type for the ArgumentParser to parse the arguments value into.
  pub fn with_type(mut self, arg_type: ArgumentType) -> Self {
    self.arg_type = arg_type;
    return self;
  }

  /// Sets the description (Help string) of the argument for
  pub fn with_description(mut self, description: &str) -> Self {
    self.description = description.to_string();
    return self;
  }

  /// Sets the default value if the argument type matches the value or panics
  pub fn with_default_value(mut self, value: ArgumentValue) -> Self {
    match (self.arg_type, value.clone()) {
      (ArgumentType::String, ArgumentValue::String(_))
      | (ArgumentType::Integer, ArgumentValue::Integer(_))
      | (ArgumentType::Float, ArgumentValue::Float(_))
      | (ArgumentType::Double, ArgumentValue::Double(_))
      | (ArgumentType::Boolean, ArgumentValue::Boolean(_)) => {
        self.default_value = value;
      }
      (_, _) => panic!(
        "Argument type: {:?} is incompatible with default value: {:?}",
        self.arg_type, value
      ),
    }

    return self;
  }

  /// Add short/long aliases (e.g., ["-o", "--output"]).
  pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
    for a in aliases {
      self.aliases.push(a.to_string());
    }
    self
  }

  /// Mark argument as positional (consumes tokens without leading -/--).
  pub fn as_positional(mut self) -> Self {
    self.positional = true;
    self
  }

  pub fn arg_type(&self) -> ArgumentType {
    return self.arg_type.clone();
  }

  pub fn name(&self) -> &str {
    return self.name.as_ref();
  }

  pub fn default_value(&self) -> ArgumentValue {
    return self.default_value.clone();
  }

  pub fn description(&self) -> &str {
    return self.description.as_ref();
  }

  pub fn aliases(&self) -> &Vec<String> {
    &self.aliases
  }
  pub fn is_positional(&self) -> bool {
    self.positional
  }
}

#[derive(Debug, Clone)]
pub struct ParsedArgument {
  name: String,
  value: ArgumentValue,
}

impl ParsedArgument {
  fn new(name: &str, value: ArgumentValue) -> Self {
    return ParsedArgument {
      name: name.to_string(),
      value,
    };
  }

  pub fn name(&self) -> String {
    return self.name.clone();
  }

  pub fn value(&self) -> ArgumentValue {
    return self.value.clone();
  }
}

impl ArgumentParser {
  /// Constructor for the argument parser.
  pub fn new(name: &str) -> Self {
    return ArgumentParser {
      name: name.to_string(),
      description: String::new(),
      authors: vec![],
      args: HashMap::new(),
      aliases: HashMap::new(),
      positionals: vec![],
      env_prefix: None,
      ignore_unknown: false,
      exclusive_groups: vec![],
      requires: vec![],
      config_path: None,
      subcommands: HashMap::new(),
      is_subcommand: false,
    };
  }

  /// The name of the parser.
  pub fn name(&self) -> &str {
    return self.name.as_ref();
  }

  /// The number of arguments currently registered with the parser.
  pub fn argument_count(&self) -> usize {
    return self.args.len();
  }

  pub fn with_author(mut self, author: &str) -> Self {
    self.authors.push(author.to_string());
    self
  }

  // TODO(vmarcella): Add description to the name
  pub fn with_description(mut self, description: &str) -> Self {
    self.description = description.to_string();
    self
  }

  pub fn with_argument(mut self, argument: Argument) -> Self {
    let idx = self.args.len();
    let name = argument.name().to_string();
    if argument.is_positional() {
      self.positionals.push(name.clone());
    }
    for a in argument.aliases().iter() {
      self.aliases.insert(a.clone(), name.clone());
    }
    self.args.insert(name, (argument, false, idx));
    return self;
  }

  /// Set an environment variable prefix (e.g., "OBJ_LOADER").
  pub fn with_env_prefix(mut self, prefix: &str) -> Self {
    self.env_prefix = Some(prefix.to_string());
    self
  }

  /// Ignore unknown arguments instead of erroring.
  pub fn ignore_unknown(mut self, ignore: bool) -> Self {
    self.ignore_unknown = ignore;
    self
  }

  /// Add a mutually exclusive group. Provide canonical names (e.g., "--a").
  pub fn with_exclusive_group(mut self, group: &[&str]) -> Self {
    self
      .exclusive_groups
      .push(group.iter().map(|s| s.to_string()).collect());
    self
  }

  /// Add a requires relationship: if `name` is present, `requires` must be.
  pub fn with_requires(mut self, name: &str, requires: &str) -> Self {
    self.requires.push((name.to_string(), requires.to_string()));
    self
  }

  /// Merge values from a simple key=value config file (optional).
  pub fn with_config_file(mut self, path: &str) -> Self {
    self.config_path = Some(path.to_string());
    self
  }

  /// Add a subcommand parser.
  pub fn with_subcommand(mut self, mut sub: ArgumentParser) -> Self {
    sub.is_subcommand = true;
    let key = sub.name.clone();
    self.subcommands.insert(key, sub);
    self
  }

  /// Generate a usage string for the parser based on registered arguments.
  pub fn usage(&self) -> String {
    let mut out = String::new();
    if self.subcommands.is_empty() {
      out.push_str(&format!("Usage: {} [options]", self.name));
    } else {
      out.push_str(&format!("Usage: {} <subcommand> [options]", self.name));
    }
    if !self.positionals.is_empty() {
      for p in &self.positionals {
        out.push_str(&format!(" <{}>", normalize_name_display(p)));
      }
    }
    out.push('\n');
    if !self.description.is_empty() {
      out.push_str(&format!("\n{}\n", self.description));
    }
    if !self.authors.is_empty() {
      out.push_str(&format!("\nAuthor(s): {}\n", self.authors.join(", ")));
    }
    out.push_str("\nOptions:\n");
    // stable iteration by index order
    let mut items: Vec<(&Argument, bool, usize)> =
      self.args.values().map(|(a, f, i)| (a, *f, *i)).collect();
    items.sort_by_key(|(_, _, i)| *i);
    for (arg, _found, _idx) in items {
      let req = if arg.required { " (required)" } else { "" };
      let def = match arg.default_value() {
        ArgumentValue::None => String::new(),
        ArgumentValue::String(s) => format!(" [default: {}]", s),
        ArgumentValue::Integer(i) => format!(" [default: {}]", i),
        ArgumentValue::Float(v) => format!(" [default: {}]", v),
        ArgumentValue::Double(v) => format!(" [default: {}]", v),
        ArgumentValue::Boolean(b) => format!(" [default: {}]", b),
      };
      let ty = match arg.arg_type() {
        ArgumentType::String => "<string>",
        ArgumentType::Integer => "<int>",
        ArgumentType::Float => "<float>",
        ArgumentType::Double => "<double>",
        ArgumentType::Boolean => "<bool>",
        ArgumentType::Count => "(count)",
        ArgumentType::StringList => "<string>...",
        ArgumentType::IntegerList => "<int>...",
        ArgumentType::FloatList => "<float>...",
        ArgumentType::DoubleList => "<double>...",
      };
      let sep = if matches!(
        arg.arg_type(),
        ArgumentType::Boolean | ArgumentType::Count
      ) {
        String::new()
      } else {
        format!(" {}", ty)
      };
      let desc = arg.description();
      let aliases = if arg.aliases().is_empty() {
        String::new()
      } else {
        format!(" (aliases: {})", arg.aliases().join(", "))
      };
      out.push_str(&format!(
        "  {}{}\n      {}{}{}\n",
        arg.name(),
        sep,
        desc,
        format!("{}{}", req, def),
        aliases
      ));
    }
    if !self.subcommands.is_empty() {
      out.push_str("\nSubcommands:\n");
      let mut keys: Vec<_> = self.subcommands.keys().cloned().collect();
      keys.sort();
      for k in keys {
        out.push_str(&format!("  {}\n", k));
      }
    }
    out
  }

  /// New non-panicking parser. Prefer this over `compile`.
  pub fn parse(mut self, args: &[String]) -> Result<ParsedArgs, ArgsError> {
    // Errors are returned, not panicked.
    let mut collecting_values = false;
    let mut last_key: Option<String> = None;

    let mut parsed_arguments = vec![];
    parsed_arguments.resize(
      self.args.len(),
      ParsedArgument::new("", ArgumentValue::None),
    );

    // Build a helper to map `--no-flag` to `--flag` when boolean
    let mut name_to_bool: HashMap<String, bool> = HashMap::new();

    let mut iter = args.iter();
    // skip executable name
    iter.next();
    // subcommand dispatch (first non-dash token)
    if let Some(token) = iter.clone().next() {
      let t = token.as_str();
      if !t.starts_with('-') && self.subcommands.contains_key(t) {
        // reconstruct argv for subcommand: exe, rest (exclude subcommand token)
        let mut argv = Vec::new();
        argv.push(args[0].clone());
        for s in args.iter().skip(2) {
          argv.push(s.clone());
        }
        let sub = self.subcommands.remove(t).unwrap();
        let sub_parsed = sub.parse(&argv)?;
        return Ok(ParsedArgs {
          values: vec![],
          subcommand: Some((t.to_string(), Box::new(sub_parsed))),
        });
      }
    }
    while let Some(token) = iter.next() {
      let arg_token = token.as_str();

      // If first non-dash token matches a subcommand, delegate here as well.
      if !arg_token.starts_with('-') && self.subcommands.contains_key(arg_token)
      {
        let mut argv2 = vec![args[0].clone()];
        for s in iter.clone() {
          argv2.push(s.to_string());
        }
        let sub = self.subcommands.remove(arg_token).unwrap();
        let sub_parsed = sub.parse(&argv2)?;
        return Ok(ParsedArgs {
          values: vec![],
          subcommand: Some((arg_token.to_string(), Box::new(sub_parsed))),
        });
      }

      if arg_token == "--help" || arg_token == "-h" {
        return Err(ArgsError::HelpRequested(self.usage()));
      }

      if arg_token == "--" {
        for value in iter.by_ref() {
          self.assign_next_positional(&mut parsed_arguments, value.as_str())?;
        }
        break;
      }

      // Handle --no-flag
      if let Some(stripped) = arg_token.strip_prefix("--no-") {
        let key = match self.canonical_name(&format!("--{}", stripped)) {
          Some(k) => k,
          None => {
            let msg = unknown_with_suggestion(arg_token, &self);
            if self.ignore_unknown {
              continue;
            }
            return Err(ArgsError::UnknownArgument(msg));
          }
        };
        let found = self.args.get_mut(&key).unwrap();
        if !matches!(found.0.arg_type(), ArgumentType::Boolean) {
          return Err(ArgsError::InvalidValue {
            name: found.0.name.clone(),
            expected: "boolean".to_string(),
            value: "<implicit false>".to_string(),
          });
        }
        if found.1 {
          return Err(ArgsError::DuplicateArgument(found.0.name.clone()));
        }
        let _ = name_to_bool.insert(found.0.name.clone(), false);
        let index = found.2;
        parsed_arguments[index] = ParsedArgument::new(
          found.0.name.as_str(),
          ArgumentValue::Boolean(false),
        );
        found.1 = true;
        collecting_values = false;
        last_key = None;
        continue;
      }

      // Handle --arg=value
      if let Some((key_raw, value)) = arg_token.split_once('=') {
        let key = match self.canonical_name(key_raw) {
          Some(k) => k,
          None => {
            let msg = unknown_with_suggestion(key_raw, &self);
            if self.ignore_unknown {
              continue;
            }
            return Err(ArgsError::UnknownArgument(msg));
          }
        };
        let found = self.args.get_mut(&key).unwrap();
        if found.1 {
          return Err(ArgsError::DuplicateArgument(found.0.name.clone()));
        }
        let parsed_value = parse_value(&found.0, value)?;
        let index = found.2;
        parsed_arguments[index] =
          ParsedArgument::new(found.0.name.as_str(), parsed_value);
        found.1 = true;
        collecting_values = false;
        last_key = None;
        continue;
      }

      if collecting_values {
        let key = last_key.as_ref().unwrap().clone();
        let found = self.args.get_mut(&key).expect("argument vanished");
        let parsed_value = parse_value(&found.0, arg_token)?;
        let index = found.2;
        parsed_arguments[index] =
          ParsedArgument::new(found.0.name.as_str(), parsed_value);
        collecting_values = false;
        found.1 = true;
        last_key = None;
        continue;
      }

      // Handle combined short flags like -vvv
      if arg_token.starts_with('-')
        && !arg_token.starts_with("--")
        && arg_token.len() > 2
      {
        let chars: Vec<char> = arg_token[1..].chars().collect();
        for ch in chars {
          let alias = format!("-{}", ch);
          let Some(canon) = self.canonical_name(&alias) else {
            let msg = unknown_with_suggestion(&alias, &self);
            if self.ignore_unknown {
              continue;
            }
            return Err(ArgsError::UnknownArgument(msg));
          };
          let pre = self.args.get(&canon).unwrap();
          if matches!(pre.0.arg_type(), ArgumentType::Count) {
            let index = pre.2;
            let current = match &parsed_arguments[index].value {
              ArgumentValue::Integer(v) => *v as i64,
              _ => 0,
            };
            let found = self.args.get_mut(&canon).unwrap();
            parsed_arguments[index] = ParsedArgument::new(
              found.0.name.as_str(),
              ArgumentValue::Integer((current + 1) as i64),
            );
            found.1 = true;
          } else if matches!(pre.0.arg_type(), ArgumentType::Boolean) {
            let index = pre.2;
            let found = self.args.get_mut(&canon).unwrap();
            parsed_arguments[index] = ParsedArgument::new(
              found.0.name.as_str(),
              ArgumentValue::Boolean(true),
            );
            found.1 = true;
          } else {
            return Err(ArgsError::InvalidValue {
              name: pre.0.name.clone(),
              expected: "separate value (not clustered)".to_string(),
              value: arg_token.to_string(),
            });
          }
        }
        continue;
      }

      let Some(canon_name) = self.canonical_name(arg_token) else {
        let msg = unknown_with_suggestion(arg_token, &self);
        if self.ignore_unknown {
          continue;
        }
        return Err(ArgsError::UnknownArgument(msg));
      };
      let pre = self.args.get(&canon_name).unwrap();
      if pre.1 == true {
        return Err(ArgsError::DuplicateArgument(pre.0.name.clone()));
      }
      // Boolean flags can be set by presence alone
      if matches!(pre.0.arg_type(), ArgumentType::Boolean) {
        let index = pre.2;
        parsed_arguments[index] = ParsedArgument::new(
          pre.0.name.as_str(),
          ArgumentValue::Boolean(true),
        );
        let found = self.args.get_mut(&canon_name).unwrap();
        found.1 = true;
        continue;
      } else if matches!(pre.0.arg_type(), ArgumentType::Count) {
        let index = pre.2;
        let found = self.args.get_mut(&canon_name).unwrap();
        let current = match &parsed_arguments[index].value {
          ArgumentValue::Integer(v) => *v as i64,
          _ => 0,
        };
        parsed_arguments[index] = ParsedArgument::new(
          found.0.name.as_str(),
          ArgumentValue::Integer((current + 1) as i64),
        );
        found.1 = true;
        continue;
      }

      collecting_values = true;
      last_key = Some(canon_name);
    }

    // If still collecting a value, it's a missing value error
    if let Some(key) = last_key {
      let arg_ref = &self.args.get(&key).unwrap().0;
      if !matches!(arg_ref.arg_type(), ArgumentType::Boolean) {
        return Err(ArgsError::MissingValue(arg_ref.name.clone()));
      }
    }

    // Attempt env var merge for missing args
    if let Some(prefix) = &self.env_prefix {
      let mut missing: Vec<String> = Vec::new();
      for (arg, found, _index) in self.args.values() {
        if !*found {
          missing.push(arg.name.clone());
        }
      }
      for name in missing {
        if let Some(val) = read_env_var(prefix, name.as_str()) {
          if let Some((arg, _found, index)) = self.args.get(name.as_str()) {
            if let Ok(parsed) = parse_value(arg, &val) {
              parsed_arguments[*index] =
                ParsedArgument::new(arg.name.as_str(), parsed);
              if let Some(entry) = self.args.get_mut(name.as_str()) {
                entry.1 = true;
              }
            }
          }
        }
      }
    }

    // Config file merge (simple key=value)
    if let Some(path) = &self.config_path {
      if let Ok(file_vals) = read_config_file(path, &self) {
        // collect names where not found yet
        let mut not_found = std::collections::HashSet::new();
        for (_n, (arg, found, _)) in self.args.iter() {
          if !*found {
            not_found.insert(arg.name.clone());
          }
        }
        for (name, raw) in file_vals {
          if !not_found.contains(&name) {
            continue;
          }
          if let Some((arg, _found, index)) = self.args.get(&name) {
            if let Ok(parsed) = parse_value(arg, &raw) {
              parsed_arguments[*index] =
                ParsedArgument::new(arg.name.as_str(), parsed);
              if let Some(e) = self.args.get_mut(name.as_str()) {
                e.1 = true;
              }
            }
          }
        }
      }
    }

    // Fill defaults and check required
    for (arg, found, index) in self.args.values() {
      match (arg.required, found, arg.default_value.clone()) {
        (true, false, _) => {
          return Err(ArgsError::MissingRequired(arg.name.clone()))
        }
        (false, false, value) => {
          parsed_arguments[*index] =
            ParsedArgument::new(arg.name.as_str(), value);
        }
        _ => {}
      }
    }

    // Validate requires and exclusive groups
    for (name, req) in &self.requires {
      let a = self.get_present(&parsed_arguments, name);
      let b = self.get_present(&parsed_arguments, req);
      if a && !b {
        return Err(ArgsError::InvalidValue {
          name: name.clone(),
          expected: format!("requires {}", req),
          value: String::new(),
        });
      }
    }
    for group in &self.exclusive_groups {
      let mut count = 0;
      for n in group {
        if self.get_present(&parsed_arguments, n) {
          count += 1;
        }
      }
      if count > 1 {
        return Err(ArgsError::InvalidValue {
          name: group.join(", "),
          expected: "mutually exclusive (choose one)".to_string(),
          value: "multiple provided".to_string(),
        });
      }
    }

    Ok(ParsedArgs {
      values: parsed_arguments,
      subcommand: None,
    })
  }

  /// Backwards-compatible panicking API. Prefer `parse` for non-panicking behavior.
  pub fn compile(self, args: &[String]) -> Vec<ParsedArgument> {
    match self.parse(args) {
      Ok(parsed) => parsed.into_vec(),
      Err(err) => panic!("{}", err),
    }
  }
}

fn parse_value(arg: &Argument, raw: &str) -> Result<ArgumentValue, ArgsError> {
  match arg.arg_type() {
    ArgumentType::String => Ok(ArgumentValue::String(raw.to_string())),
    ArgumentType::Float => raw
      .parse::<f32>()
      .map(ArgumentValue::Float)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "float".to_string(),
        value: raw.to_string(),
      }),
    ArgumentType::Double => raw
      .parse::<f64>()
      .map(ArgumentValue::Double)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "double".to_string(),
        value: raw.to_string(),
      }),
    ArgumentType::Integer => raw
      .parse::<i64>()
      .map(ArgumentValue::Integer)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "integer".to_string(),
        value: raw.to_string(),
      }),
    ArgumentType::Boolean => raw
      .parse::<bool>()
      .map(ArgumentValue::Boolean)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "boolean".to_string(),
        value: raw.to_string(),
      }),
    ArgumentType::Count => Ok(ArgumentValue::Integer(1)),
    ArgumentType::StringList => Ok(ArgumentValue::String(raw.to_string())),
    ArgumentType::IntegerList => raw
      .parse::<i64>()
      .map(ArgumentValue::Integer)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "integer".to_string(),
        value: raw.to_string(),
      }),
    ArgumentType::FloatList => raw
      .parse::<f32>()
      .map(ArgumentValue::Float)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "float".to_string(),
        value: raw.to_string(),
      }),
    ArgumentType::DoubleList => raw
      .parse::<f64>()
      .map(ArgumentValue::Double)
      .map_err(|_| ArgsError::InvalidValue {
        name: arg.name.clone(),
        expected: "double".to_string(),
        value: raw.to_string(),
      }),
  }
}

#[derive(Debug)]
pub enum ArgsError {
  UnknownArgument(String),
  DuplicateArgument(String),
  MissingValue(String),
  InvalidValue {
    name: String,
    expected: String,
    value: String,
  },
  MissingRequired(String),
  HelpRequested(String),
}

impl fmt::Display for ArgsError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ArgsError::UnknownArgument(a) => write!(f, "Unknown argument: {}", a),
      ArgsError::DuplicateArgument(a) => write!(f, "Duplicate argument: {}", a),
      ArgsError::MissingValue(a) => {
        write!(f, "Missing value for argument: {}", a)
      }
      ArgsError::InvalidValue {
        name,
        expected,
        value,
      } => write!(
        f,
        "Invalid value for {}: got '{}', expected {}",
        name, value, expected
      ),
      ArgsError::MissingRequired(a) => {
        write!(f, "Missing required argument: {}", a)
      }
      ArgsError::HelpRequested(usage) => write!(f, "{}", usage),
    }
  }
}

impl std::error::Error for ArgsError {}

/// A parsed arguments wrapper with typed getters.
#[derive(Debug, Clone)]
pub struct ParsedArgs {
  values: Vec<ParsedArgument>,
  subcommand: Option<(String, Box<ParsedArgs>)>,
}

impl ParsedArgs {
  pub fn into_vec(self) -> Vec<ParsedArgument> {
    self.values
  }

  pub fn has(&self, name: &str) -> bool {
    self
      .values
      .iter()
      .any(|p| p.name == name && !matches!(p.value, ArgumentValue::None))
  }

  pub fn get_string(&self, name: &str) -> Option<String> {
    self
      .values
      .iter()
      .find(|p| p.name == name)
      .and_then(|p| match &p.value {
        ArgumentValue::String(s) => Some(s.clone()),
        _ => None,
      })
  }

  pub fn get_i64(&self, name: &str) -> Option<i64> {
    self
      .values
      .iter()
      .find(|p| p.name == name)
      .and_then(|p| match &p.value {
        ArgumentValue::Integer(v) => Some(*v),
        _ => None,
      })
  }

  pub fn get_f32(&self, name: &str) -> Option<f32> {
    self
      .values
      .iter()
      .find(|p| p.name == name)
      .and_then(|p| match &p.value {
        ArgumentValue::Float(v) => Some(*v),
        _ => None,
      })
  }

  pub fn get_f64(&self, name: &str) -> Option<f64> {
    self
      .values
      .iter()
      .find(|p| p.name == name)
      .and_then(|p| match &p.value {
        ArgumentValue::Double(v) => Some(*v),
        _ => None,
      })
  }

  pub fn get_bool(&self, name: &str) -> Option<bool> {
    self
      .values
      .iter()
      .find(|p| p.name == name)
      .and_then(|p| match &p.value {
        ArgumentValue::Boolean(v) => Some(*v),
        _ => None,
      })
  }

  pub fn get_count(&self, name: &str) -> Option<i64> {
    self
      .values
      .iter()
      .find(|p| p.name == name)
      .and_then(|p| match &p.value {
        ArgumentValue::Integer(v) => Some(*v),
        _ => None,
      })
  }

  pub fn subcommand(&self) -> Option<(&str, &ParsedArgs)> {
    self
      .subcommand
      .as_ref()
      .map(|(n, p)| (n.as_str(), p.as_ref()))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn argv(args: &[&str]) -> Vec<String> {
    let mut v = vec!["prog".to_string()];
    v.extend(args.iter().map(|s| s.to_string()));
    v
  }

  #[test]
  fn required_and_default() {
    let parser = ArgumentParser::new("app")
      .with_argument(
        Argument::new("--name")
          .is_required(true)
          .with_type(ArgumentType::String),
      )
      .with_argument(
        Argument::new("--count")
          .with_type(ArgumentType::Integer)
          .with_default_value(ArgumentValue::Integer(2)),
      );
    let res = parser.parse(&argv(&["--name", "lambda"])).unwrap();
    assert_eq!(res.get_string("--name").unwrap(), "lambda");
    assert_eq!(res.get_i64("--count").unwrap(), 2);
  }

  #[test]
  fn help_requested() {
    let parser = ArgumentParser::new("app").with_description("desc");
    let err = parser.parse(&argv(&["--help"])).unwrap_err();
    match err {
      ArgsError::HelpRequested(u) => assert!(u.contains("Usage: app")),
      _ => panic!(),
    }
  }

  #[test]
  fn boolean_presence_and_no() {
    let parser = ArgumentParser::new("app").with_argument(
      Argument::new("--verbose").with_type(ArgumentType::Boolean),
    );
    let p = parser.parse(&argv(&["--verbose"])).unwrap();
    assert_eq!(p.get_bool("--verbose").unwrap(), true);
    let parser2 = ArgumentParser::new("app").with_argument(
      Argument::new("--verbose").with_type(ArgumentType::Boolean),
    );
    let p2 = parser2.parse(&argv(&["--no-verbose"])).unwrap();
    assert_eq!(p2.get_bool("--verbose").unwrap(), false);
  }

  #[test]
  fn equals_syntax() {
    let parser = ArgumentParser::new("app")
      .with_argument(Argument::new("--title").with_type(ArgumentType::String));
    let p = parser.parse(&argv(&["--title=Demo"])).unwrap();
    assert_eq!(p.get_string("--title").unwrap(), "Demo");
  }

  #[test]
  fn aliases_and_short() {
    let parser = ArgumentParser::new("app").with_argument(
      Argument::new("--output")
        .with_type(ArgumentType::String)
        .with_aliases(&["-o"]),
    );
    let p = parser.parse(&argv(&["-o", "file.txt"])).unwrap();
    assert_eq!(p.get_string("--output").unwrap(), "file.txt");
  }

  #[test]
  fn positional_and_terminator() {
    let parser = ArgumentParser::new("app")
      .with_argument(
        Argument::new("input")
          .as_positional()
          .with_type(ArgumentType::String),
      )
      .with_argument(
        Argument::new("rest")
          .as_positional()
          .with_type(ArgumentType::String),
      );
    let p = parser.parse(&argv(&["--", "a", "b"])).unwrap();
    assert_eq!(p.get_string("input").unwrap(), "a");
    assert_eq!(p.get_string("rest").unwrap(), "b");
  }

  #[test]
  fn counting_and_cluster() {
    let parser = ArgumentParser::new("app").with_argument(
      Argument::new("-v")
        .with_aliases(&["-v"])
        .with_type(ArgumentType::Count),
    );
    let p = parser.parse(&argv(&["-vvv"])).unwrap();
    assert_eq!(p.get_count("-v").unwrap(), 3);
  }

  #[test]
  fn env_prefix() {
    std::env::set_var("TEST_PATH", "/tmp/x");
    let parser = ArgumentParser::new("app")
      .with_env_prefix("TEST")
      .with_argument(Argument::new("--path").with_type(ArgumentType::String));
    let p = parser.parse(&argv(&[])).unwrap();
    assert_eq!(p.get_string("--path").unwrap(), "/tmp/x");
  }

  #[test]
  fn usage_includes_aliases_and_types() {
    let parser = ArgumentParser::new("u")
      .with_argument(
        Argument::new("--a")
          .with_type(ArgumentType::String)
          .with_aliases(&["-a"]),
      )
      .with_argument(Argument::new("--b").with_type(ArgumentType::Integer))
      .with_argument(Argument::new("--c").with_type(ArgumentType::Float))
      .with_argument(Argument::new("--d").with_type(ArgumentType::Double))
      .with_argument(Argument::new("--e").with_type(ArgumentType::Boolean))
      .with_argument(
        Argument::new("-v")
          .with_aliases(&["-v"])
          .with_type(ArgumentType::Count),
      )
      .with_argument(
        Argument::new("S1")
          .as_positional()
          .with_type(ArgumentType::String),
      );
    let u = parser.usage();
    assert!(u.contains("aliases: -a"));
    assert!(u.contains("<string>"));
    assert!(u.contains("<int>"));
    assert!(u.contains("<float>"));
    assert!(u.contains("<double>"));
    assert!(u.contains("--e"));
    // Count has no value placeholder in usage; ensure it exists
    assert!(u.contains("-v"));
  }

  #[test]
  fn no_flag_non_boolean_invalid() {
    let parser = ArgumentParser::new("app")
      .with_argument(Argument::new("--name").with_type(ArgumentType::String));
    let err = parser.parse(&argv(&["--no-name"])).unwrap_err();
    match err {
      ArgsError::InvalidValue { expected, .. } => {
        assert!(expected.contains("boolean"))
      }
      _ => panic!(),
    }
  }

  #[test]
  fn equals_unknown_ignore_vs_error() {
    let parser = ArgumentParser::new("app");
    let err = parser.parse(&argv(&["--unknown=1"])).unwrap_err();
    match err {
      ArgsError::UnknownArgument(_msg) => {}
      _ => panic!(),
    }

    let parser2 = ArgumentParser::new("app").ignore_unknown(true);
    let ok = parser2.parse(&argv(&["--unknown=1"])).unwrap();
    assert_eq!(ok.into_vec().len(), 0);
  }

  #[test]
  fn cluster_invalid_for_non_boolean_or_count() {
    let parser = ArgumentParser::new("app").with_argument(
      Argument::new("-o")
        .with_aliases(&["-o"])
        .with_type(ArgumentType::String),
    );
    let err = parser.parse(&argv(&["-ov"])).unwrap_err();
    match err {
      ArgsError::InvalidValue { expected, .. } => {
        assert!(expected.contains("separate value"))
      }
      _ => panic!(),
    }
  }

  #[test]
  fn unknown_argument_suggests() {
    let parser = ArgumentParser::new("app")
      .with_argument(Argument::new("--port").with_type(ArgumentType::Integer));
    let err = parser.parse(&argv(&["--portt", "1"])).unwrap_err();
    match err {
      ArgsError::UnknownArgument(msg) => assert!(msg.contains("did you mean")),
      _ => panic!(),
    }
  }

  #[test]
  fn missing_value_error() {
    let parser = ArgumentParser::new("app")
      .with_argument(Argument::new("--name").with_type(ArgumentType::String));
    let err = parser.parse(&argv(&["--name"])).unwrap_err();
    match err {
      ArgsError::MissingValue(_) | ArgsError::InvalidValue { .. } => {}
      _ => panic!("unexpected: {:?}", err),
    }
  }

  #[test]
  fn duplicate_argument_error() {
    let parser = ArgumentParser::new("app")
      .with_argument(Argument::new("--name").with_type(ArgumentType::String));
    let err = parser
      .parse(&argv(&["--name", "a", "--name", "b"]))
      .unwrap_err();
    match err {
      ArgsError::DuplicateArgument(_) => {}
      _ => panic!(),
    }
  }

  #[test]
  fn env_overridden_by_cli() {
    std::env::set_var("APP_PATH", "/env");
    let parser = ArgumentParser::new("app")
      .with_env_prefix("APP")
      .with_argument(Argument::new("--path").with_type(ArgumentType::String));
    let p = parser.parse(&argv(&["--path", "/cli"])).unwrap();
    assert_eq!(p.get_string("--path").unwrap(), "/cli");
  }

  #[test]
  fn config_merge_canonical_and_uppercase() {
    // Build config content
    let dir = std::env::temp_dir();
    let path = dir.join("args_cfg_test.cfg");
    std::fs::write(&path, "--host=1.2.3.4\nPORT=9000\n# comment\n").unwrap();
    let parser = ArgumentParser::new("app")
      .with_config_file(path.to_str().unwrap())
      .with_argument(Argument::new("--host").with_type(ArgumentType::String))
      .with_argument(Argument::new("--port").with_type(ArgumentType::Integer));
    let p = parser.parse(&argv(&[])).unwrap();
    assert_eq!(p.get_string("--host").unwrap(), "1.2.3.4");
    assert_eq!(p.get_i64("--port").unwrap(), 9000);
  }

  #[test]
  fn subcommand_parsing() {
    let root = ArgumentParser::new("tool").with_subcommand(
      ArgumentParser::new("serve").with_argument(
        Argument::new("--port").with_type(ArgumentType::Integer),
      ),
    );
    let p = root.parse(&argv(&["serve", "--port", "8081"])).unwrap();
    let (name, sub) = p.subcommand().unwrap();
    assert_eq!(name, "serve");
    assert_eq!(sub.get_i64("--port").unwrap(), 8081);
  }

  #[test]
  fn assign_next_positional_error_on_extra() {
    let parser = ArgumentParser::new("pos").with_argument(
      Argument::new("a")
        .as_positional()
        .with_type(ArgumentType::String),
    );
    let err = parser.parse(&argv(&["--", "x", "y"])).unwrap_err();
    match err {
      ArgsError::InvalidValue { expected, .. } => {
        assert!(expected.contains("no extra positional"))
      }
      _ => panic!(),
    }
  }

  #[test]
  fn argumentvalue_conversions_success() {
    let s: String = ArgumentValue::String("hi".into()).into();
    let i: i64 = ArgumentValue::Integer(7).into();
    let f: f32 = ArgumentValue::Float(1.5).into();
    let d: f64 = ArgumentValue::Double(2.5).into();
    assert_eq!(s, "hi");
    assert_eq!(i, 7);
    assert_eq!(f, 1.5);
    assert_eq!(d, 2.5);
  }
}

impl ArgumentParser {
  fn canonical_name(&self, token: &str) -> Option<String> {
    if self.args.contains_key(token) {
      return Some(token.to_string());
    }
    if let Some(name) = self.aliases.get(token) {
      return Some(name.clone());
    }
    None
  }

  fn assign_next_positional(
    &mut self,
    out: &mut Vec<ParsedArgument>,
    value: &str,
  ) -> Result<(), ArgsError> {
    for pname in self.positionals.clone() {
      if let Some(entry) = self.args.get_mut(&pname) {
        if entry.1 == false {
          let parsed = parse_value(&entry.0, value)?;
          let idx = entry.2;
          out[idx] = ParsedArgument::new(entry.0.name.as_str(), parsed);
          entry.1 = true;
          return Ok(());
        }
      }
    }
    Err(ArgsError::InvalidValue {
      name: "<positional>".to_string(),
      expected: "no extra positional arguments".to_string(),
      value: value.to_string(),
    })
  }

  fn get_present(&self, out: &Vec<ParsedArgument>, name: &str) -> bool {
    let canon = if self.args.contains_key(name) {
      name.to_string()
    } else if let Some(n) = self.aliases.get(name) {
      n.clone()
    } else {
      name.to_string()
    };
    if let Some((_a, _f, idx)) = self.args.get(&canon) {
      return !matches!(out[*idx].value, ArgumentValue::None);
    }
    false
  }
}

fn normalize_name_display(name: &str) -> String {
  name
    .trim_start_matches('-')
    .replace('-', "_")
    .to_uppercase()
}

fn read_env_var(prefix: &str, name: &str) -> Option<String> {
  let key = format!("{}_{}", prefix, normalize_name_display(name));
  std::env::var(&key).ok()
}

fn read_config_file(
  path: &str,
  parser: &ArgumentParser,
) -> Result<Vec<(String, String)>, ()> {
  let content = std::fs::read_to_string(path).map_err(|_| ())?;
  let mut norm: HashMap<String, String> = HashMap::new();
  for (name, (_a, _f, _i)) in parser.args.iter() {
    norm.insert(normalize_name_display(name), name.clone());
  }
  let mut out = vec![];
  for line in content.lines() {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
      continue;
    }
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
      continue;
    }
    let k = parts[0].trim();
    let v = parts[1].trim();
    let key = if parser.args.contains_key(k) {
      k.to_string()
    } else if let Some(c) = norm.get(&k.to_uppercase()) {
      c.clone()
    } else {
      continue;
    };
    out.push((key, v.to_string()));
  }
  Ok(out)
}

fn unknown_with_suggestion(arg: &str, parser: &ArgumentParser) -> String {
  let mut best: Option<(usize, String)> = None;
  for key in parser.args.keys() {
    let d = levenshtein(arg, key);
    if best.as_ref().map(|(bd, _)| d < *bd).unwrap_or(true) {
      best = Some((d, key.clone()));
    }
  }
  if let Some((_d, name)) = best {
    format!("{} (did you mean '{}'?)", arg, name)
  } else {
    arg.to_string()
  }
}

fn levenshtein(a: &str, b: &str) -> usize {
  let mut prev: Vec<usize> = (0..=b.len()).collect();
  let mut cur: Vec<usize> = vec![0; b.len() + 1];
  for (i, ca) in a.chars().enumerate() {
    cur[0] = i + 1;
    for (j, cb) in b.chars().enumerate() {
      let cost = if ca == cb { 0 } else { 1 };
      cur[j + 1] = std::cmp::min(
        std::cmp::min(cur[j] + 1, prev[j + 1] + 1),
        prev[j] + cost,
      );
    }
    std::mem::swap(&mut prev, &mut cur);
  }
  prev[b.len()]
}
