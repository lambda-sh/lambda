//! # Lambda Args
//! Lambda Args is a simple argument parser for Rust. It is designed to be
//! simple to use and primarily for use in lambda command line applications.

use std::collections::HashMap;

pub struct ArgumentParser {
  name: String,
  args: HashMap<String, (Argument, bool, usize)>,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub enum ArgumentType {
  Boolean,
  Integer,
  Float,
  Double,
  String,
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
      | (ArgumentType::Double, ArgumentValue::Double(_)) => {
        self.default_value = value;
      }
      (_, _) => panic!(
        "Argument type: {:?} is incompatible with default value: {:?}",
        self.arg_type, value
      ),
    }

    return self;
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
      args: HashMap::new(),
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

  pub fn with_author(mut self, author: &str) {
    todo!("Implement adding authors to a command line parser")
  }

  // TODO(vmarcella): Add description to the name
  pub fn with_description(mut self, description: &str) {
    todo!("Implement adding a description to the command line parser.")
  }

  pub fn with_argument(mut self, argument: Argument) -> Self {
    self.args.insert(
      argument.name().to_string(),
      (argument, false, self.args.len()),
    );
    return self;
  }

  /// Compiles a slice of Strings into an array of Parsed Arguments. This will
  /// move the parser into this function and return back the parsed arguments if
  /// everything succeeds. This function assumes that the first item within args
  /// is the name of the executable being run. (Which is the standard for
  /// arguments passed in from std::env::args()). The ordering of the arguments
  /// returned is always the same as order they're registered in with the
  /// parser.
  pub fn compile(mut self, args: &[String]) -> Vec<ParsedArgument> {
    let mut collecting_values = false;
    let mut last_argument: Option<&mut (Argument, bool, usize)> = None;

    let mut parsed_arguments = vec![];
    parsed_arguments.resize(
      self.args.len(),
      ParsedArgument::new("", ArgumentValue::None),
    );

    for arg in args.into_iter().skip(1) {
      if collecting_values {
        let (arg_ref, found, index) = last_argument.as_mut().unwrap();

        let parsed_value = match arg_ref.arg_type() {
          ArgumentType::String => ArgumentValue::String(arg.clone()),
          ArgumentType::Float => {
            ArgumentValue::Float(arg.parse().unwrap_or_else(|err| {
              panic!(
                "Could not convert {:?} to a float because of: {}",
                arg, err
              )
            }))
          }
          ArgumentType::Double => {
            ArgumentValue::Double(arg.parse().unwrap_or_else(|err| {
              panic!(
                "Could not convert {:?} to a double because of: {}",
                arg, err
              )
            }))
          }
          ArgumentType::Integer => {
            ArgumentValue::Integer(arg.parse().unwrap_or_else(|err| {
              panic!(
                "Could not convert {:?} to an integer because of: {}",
                arg, err
              )
            }))
          }
          ArgumentType::Boolean => {
            ArgumentValue::Boolean(arg.parse().unwrap_or_else(|err| {
              panic!(
                "Could not convert {:?} to a boolean because of: {}",
                arg, err
              )
            }))
          }
        };

        parsed_arguments[*index] =
          ParsedArgument::new(arg_ref.name.as_str(), parsed_value);

        collecting_values = false;
        *found = true;
        continue;
      }

      // Panic if the argument cannot be found inside of the registered
      // arguments.
      let found_argument = self.args.get_mut(arg).unwrap_or_else(|| {
        panic!("Argument: {} is not a valid argument", &arg)
      });

      // If the argument has already been found, throw an error.
      if found_argument.1 == true {
        panic!("{} was set more than once.", found_argument.0.name.clone());
      }

      collecting_values = true;
      last_argument = Some(found_argument);
    }

    // Go through all of the registered arguments and check for forgotten flags/
    // apply default values.
    for (arg, found, index) in self.args.values() {
      match (arg.required, found, arg.default_value.clone()) {
        // Argument was required as user input, but not found.
        (true, false, _) => panic!(
          "--{} is a required argument, but was not found.",
          arg.name.clone()
        ),
        // Argument wasn't required & wasn't found, but has a default value
        (false, false, value) => {
          parsed_arguments[*index] =
            ParsedArgument::new(arg.name.as_str(), value);
        }
        // Any other situation doesn't really matter and will be a noop
        (_, _, _) => {}
      }
    }
    return parsed_arguments;
  }
}
