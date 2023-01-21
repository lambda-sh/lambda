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
  obj_path: String,
}

impl Into<Args> for Vec<ParsedArgument> {
  fn into(self) -> Args {
    let mut args = Args {
      obj_path: String::new(),
    };

    for arg in self {
      match (arg.name().as_str(), arg.value()) {
        ("--obj-path", ArgumentValue::String(path)) => args.obj_path = path,
        (_, _) => {}
      }
    }

    return args;
  }
}

fn parse_arguments() -> Args {
  let parser = ArgumentParser::new("obj-loader");

  let obj_file = Argument::new("--obj-path")
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

  let mesh = MeshBuilder::new().build_from_obj(&args.obj_path);

  println!("Hello, world!");
}
