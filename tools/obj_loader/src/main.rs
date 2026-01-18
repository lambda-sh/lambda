#![allow(clippy::needless_return)]

use std::env;

use args::{
  Argument,
  ArgumentParser,
  ArgumentType,
  ArgumentValue,
  ParsedArgument,
};
use lambda::{
  component::Component,
  events::{
    EventMask,
    WindowEvent,
  },
  logging,
  render::{
    buffer::BufferBuilder,
    command::RenderCommand,
    mesh::{
      Mesh,
      MeshBuilder,
    },
    pipeline::RenderPipelineBuilder,
    render_pass::RenderPassBuilder,
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    viewport,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

// ------------------------------ SHADER SOURCE --------------------------------

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;
layout (location = 1) out vec3 frag_normal;

layout ( push_constant ) uniform PushConstant {
  vec4 data;
  mat4 render_matrix;
} push_constants;

void main() {
  gl_Position = push_constants.render_matrix * vec4(vertex_position, 1.0);
  frag_color = vertex_color;
  frag_normal = vertex_normal;
}

"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 frag_color;
layout (location = 1) in vec3 frag_normal;

layout (location = 0) out vec4 fragment_color;

void main() {
  float diffuse = dot(frag_normal, vec3(0.0, 0.0, 1.0));
  fragment_color = vec4(frag_color, 1.0) * vec4(diffuse);
}

"#;

// -------------------------------- IMMEDIATES ---------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateData {
  data: [f32; 4],
  render_matrix: [[f32; 4]; 4],
}

pub fn immediate_data_to_bytes(immediate_data: &ImmediateData) -> &[u32] {
  let bytes = unsafe {
    let size_in_bytes = std::mem::size_of::<ImmediateData>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = immediate_data as *const ImmediateData as *const u32;
    std::slice::from_raw_parts(ptr, size_in_u32)
  };

  return bytes;
}

fn make_transform(
  translate: [f32; 3],
  angle: f32,
  scale: f32,
) -> [[f32; 4]; 4] {
  let c = angle.cos() * scale;
  let s = angle.sin() * scale;

  let [x, y, z] = translate;

  return [
    [c, 0.0, s, 0.0],
    [0.0, scale, 0.0, 0.0],
    [-s, 0.0, c, 0.0],
    [x, y, z, 1.0],
  ];
}

struct Args {
  obj_path: String,
}

impl From<Vec<ParsedArgument>> for Args {
  fn from(arguments: Vec<ParsedArgument>) -> Args {
    let mut args = Args {
      obj_path: String::new(),
    };

    for arg in arguments {
      if let ("--obj-path", ArgumentValue::String(path)) =
        (arg.name().as_str(), arg.value())
      {
        args.obj_path = path;
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

  return Args::from(args);
}

struct ObjLoader {
  obj_path: String,
  vertex_shader: Shader,
  fragment_shader: Shader,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
  mesh: Option<Mesh>,
  frame_number: u32,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for ObjLoader {
  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    if let WindowEvent::Resize { width, height } = event {
      self.width = *width;
      self.height = *height;
      logging::info!("Window resized to {}x{}", width, height);
    }
    return Ok(());
  }

  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    let gpu = render_context.gpu();
    let surface_format = render_context.surface_format();
    let depth_format = render_context.depth_format();

    let render_pass =
      RenderPassBuilder::new().build(gpu, surface_format, depth_format);
    let immediate_data_size = std::mem::size_of::<ImmediateData>() as u32;

    let mesh = MeshBuilder::new().build_from_obj(&self.obj_path);

    logging::trace!(
      "[DEBUG] Mesh data from {} Mesh:\n {:#?}",
      &self.obj_path,
      mesh
    );

    let pipeline = RenderPipelineBuilder::new()
      .with_immediate_data(immediate_data_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&mesh, gpu)
          .expect("Failed to create buffer"),
        mesh.attributes().to_vec(),
      )
      .build(
        gpu,
        surface_format,
        depth_format,
        &render_pass,
        &self.vertex_shader,
        Some(&self.fragment_shader),
      );

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
    self.mesh = Some(mesh);
    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.frame_number += 1;
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<lambda::render::command::RenderCommand> {
    let mesh_matrix =
      make_transform([0.0, 0.0, 0.5], self.frame_number as f32 * 0.01, 0.5);

    // Create viewport.
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    let render_pipeline = self
      .render_pipeline
      .expect("No render pipeline actively set for rendering.");

    return vec![
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetPipeline {
        pipeline: render_pipeline,
      },
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass
          .expect("Cannot begin the render pass when it doesn't exist."),
        viewport: viewport.clone(),
      },
      RenderCommand::BindVertexBuffer {
        pipeline: render_pipeline,
        buffer: 0,
      },
      RenderCommand::Immediates {
        pipeline: render_pipeline,
        offset: 0,
        bytes: Vec::from(immediate_data_to_bytes(&ImmediateData {
          data: [0.0, 0.0, 0.0, 0.0],
          render_matrix: mesh_matrix,
        })),
      },
      RenderCommand::Draw {
        vertices: 0..self.mesh.as_ref().unwrap().vertices().len() as u32,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for ObjLoader {
  fn default() -> Self {
    let virtual_vertex_shader = VirtualShader::Source {
      source: String::from(VERTEX_SHADER_SOURCE),
      kind: ShaderKind::Vertex,
      name: String::from("obj-loader-vertex"),
      entry_point: String::from("main"),
    };
    let vs = ShaderBuilder::new().build(virtual_vertex_shader);

    let virtual_fragment_shader = VirtualShader::Source {
      source: String::from(FRAGMENT_SHADER_SOURCE),
      kind: ShaderKind::Fragment,
      name: String::from("obj-loader-fragment"),
      entry_point: String::from("main"),
    };

    let fs = ShaderBuilder::new().build(virtual_fragment_shader);

    return Self {
      obj_path: String::new(),
      vertex_shader: vs,
      fragment_shader: fs,
      render_pipeline: None,
      render_pass: None,
      mesh: None,
      width: 800,
      height: 600,
      frame_number: 0,
    };
  }
}

fn main() {
  let args = parse_arguments();
  let runtime = ApplicationRuntimeBuilder::new(
    std::format!("obj-loader: {}", &args.obj_path).as_str(),
  )
  .with_window_configured_as(move |window_builder| {
    return window_builder
      .with_dimensions(800, 600)
      .with_name(std::format!("obj-loader: {}", &args.obj_path).as_str())
      .with_vsync(true);
  })
  .with_renderer_configured_as(|renderer_builder| {
    return renderer_builder.with_render_timeout(1_000_000_000);
  })
  .with_component(move |runtime, mut obj_loader: ObjLoader| {
    obj_loader.obj_path = parse_arguments().obj_path.clone();
    return (runtime, obj_loader);
  })
  .build();

  start_runtime(runtime);
}
