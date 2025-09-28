//! Minimal helpers for loading Wavefront OBJ assets.
//!
//! These functions are thin wrappers around the `obj` crate used by examples
//! and tooling to import meshes.
use std::{fs::File, io::BufReader};

use obj::{load_obj, Obj, TexturedVertex};

/// Loads a untextured obj file from the given path. Wrapper around the obj crate.
pub fn load_obj_from_file(path: &str) -> Obj {
  let file = File::open(path).unwrap();
  let reader = BufReader::new(file);
  let obj = load_obj(reader).expect("Failed to load the OBJ file.");
  return obj;
}

/// Loads a textured obj file from the given path. Wrapper around the obj crate.
pub fn load_textured_obj_from_file(path: &str) -> Obj<TexturedVertex> {
  let file = File::open(path).unwrap();
  let reader = BufReader::new(file);
  let obj = load_obj(reader).expect("Failed to load obj file.");
  return obj;
}
