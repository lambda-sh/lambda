use std::env;

use args::{
  Argument,
  ArgumentParser,
  ArgumentType,
  ArgumentValue,
  ParsedArgument,
};
use lambda::render::{
  mesh::MeshBuilder,
  vertex::{
    Vertex,
    VertexAttribute,
    VertexElement,
  },
};

struct Args {
  obj_file: String,
}

impl Into<Args> for Vec<ParsedArgument> {
  fn into(self) -> Args {
    let mut args = Args {
      obj_file: String::new(),
    };

    for arg in self {
      match (arg.name().as_str(), arg.value()) {
        ("--obj-file", ArgumentValue::String(path)) => args.obj_file = path,
        (_, _) => {}
      }
    }

    return args;
  }
}

fn parse_arguments() -> Args {
  let parser = ArgumentParser::new("obj-loader");

  let obj_file = Argument::new("--obj-file")
    .is_required(true)
    .with_type(ArgumentType::String);

  let args = parser
    .with_argument(obj_file)
    .compile(&env::args().collect::<Vec<_>>());

  return args.into();
}

struct ObjLoader {
  mesh_builder: MeshBuilder,
}

fn main() {
  let args = parse_arguments();

  let mesh = MeshBuilder::new().build_from_obj(&args.obj_file);

  println!("Hello, world!");
}
