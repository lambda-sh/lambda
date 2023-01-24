# obj-loader (WIP)
Tool to render obj files with lambda.

## Usage

To run the obj-loader, you can execute the following command:

```bash
cargo run --bin obj-loader -- --obj-path <path>
```

This will load the obj file at the given path and render it with lambda. 
Currently, only the vertices and faces are loaded, so the obj file must 
contain only triangles.


