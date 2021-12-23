use std::{env, path::Path};

fn copy_assets(dir: &Path, out_path: &str) -> std::io::Result<()> {
  if dir.is_dir() {
    for entry in std::fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_dir() {
        copy_assets(&path, out_path);
      } else {
        std::fs::copy(entry.path(), Path::new(out_path));
      }
    }
  }
  return Ok(());
}

fn main() {
  let out_dir = env::var_os("OUT_DIR").unwrap();
  let assets = std::path::Path::new("assets");
  copy_assets(assets, out_dir.to_str().unwrap());
  println!("OutDir: {}", out_dir.to_str().unwrap());
}
