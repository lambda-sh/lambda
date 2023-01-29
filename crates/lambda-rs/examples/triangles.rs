use lambda::{
  core::{
    component::Component,
    events::{
      Events,
      KeyEvent,
      VirtualKey,
      WindowEvent,
    },
    runtime::start_runtime,
  },
  render::{
    command::RenderCommand,
    pipeline::{
      self,
      PipelineStage,
    },
    render_pass,
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    viewport,
    RenderContext,
  },
  runtimes::ApplicationRuntimeBuilder,
};

pub struct TrianglesComponent {
  triangle_vertex: Shader,
  vertex_shader: Shader,
  render_pass: Option<lambda::render::ResourceId>,
  render_pipeline: Option<lambda::render::ResourceId>,
  width: u32,
  height: u32,
  animation_scalar: f32,
  position: (f32, f32),
}

impl Component for TrianglesComponent {
  fn on_attach(&mut self, render_context: &mut RenderContext) {
    let render_pass =
      render_pass::RenderPassBuilder::new().build(&render_context);

    let push_constants_size = std::mem::size_of::<PushConstant>() as u32;
    let pipeline = pipeline::RenderPipelineBuilder::new()
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .build(
        render_context,
        &render_pass,
        &self.vertex_shader,
        Some(&self.triangle_vertex),
      );

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));

    println!("Attached the DemoComponent.");
  }

  fn on_detach(&mut self, _render_context: &mut RenderContext) {}

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    let (x, y) = self.position;

    let triangle_data = &[
      PushConstant {
        color: [
          1.0,
          1.0 * self.animation_scalar,
          0.5 * self.animation_scalar,
          1.0,
        ],
        pos: [x, y],
        scale: [0.3, 0.3],
      },
      PushConstant {
        color: [0.0, 1.0, 0.0, 1.0],
        pos: [0.5, 0.0],
        scale: [0.4, 0.4],
      },
      PushConstant {
        color: [0.0, 0.0, 1.0, 1.0],
        pos: [0.25, 0.5],
        scale: [0.5, 0.5],
      },
      PushConstant {
        color: [1.0, 1.0, 1.0, 1.0],
        pos: [0.0, 0.0],
        scale: [0.5, 0.5],
      },
    ];

    let render_pipeline = self
      .render_pipeline
      .expect("No render pipeline actively set for rendering.");

    let mut commands = vec![
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetPipeline {
        pipeline: render_pipeline.clone(),
      },
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass
          .expect("Cannot begin the render pass when it doesn't exist.")
          .clone(),
        viewport: viewport.clone(),
      },
    ];

    // Upload triangle data into the the GPU at the vertex stage of the pipeline
    // before requesting to draw each triangle.
    for triangle in triangle_data {
      commands.push(RenderCommand::PushConstants {
        pipeline: render_pipeline.clone(),
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_bytes(triangle)),
      });
      commands.push(RenderCommand::Draw { vertices: 0..3 });
    }

    commands.push(RenderCommand::EndRenderPass);

    return commands;
  }

  fn on_event(&mut self, event: Events) {
    match event {
      Events::Runtime { event, issued_at } => match event {
        lambda::core::events::RuntimeEvent::Shutdown => {
          println!("Shutting down the runtime");
        }
        _ => {}
      },
      Events::Window { event, issued_at } => match event {
        WindowEvent::Resize { width, height } => {
          println!("Window resized to {}x{}", width, height);
          self.width = width;
          self.height = height;
        }
        WindowEvent::Close => {
          println!("Window closed");
        }
      },
      Events::Component { event, issued_at } => todo!(),
      Events::Keyboard { event, issued_at } => match event {
        KeyEvent::KeyPressed {
          scan_code,
          virtual_key,
        } => match virtual_key {
          Some(VirtualKey::W) => {
            self.position.1 -= 0.01;
          }
          Some(VirtualKey::S) => {
            self.position.1 += 0.01;
          }
          Some(VirtualKey::A) => {
            self.position.0 -= 0.01;
          }
          Some(VirtualKey::D) => {
            self.position.0 += 0.01;
          }
          _ => {}
        },
        _ => {}
      },
      _ => {}
    }
  }

  fn on_update(&mut self, last_frame: &std::time::Duration) {
    match last_frame.as_millis() > 20 {
      true => {
        println!("[WARN] Last frame took {}ms", last_frame.as_millis());
      }
      false => {}
    }
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PushConstant {
  color: [f32; 4],
  pos: [f32; 2],
  scale: [f32; 2],
}

pub fn push_constants_to_bytes(push_constants: &PushConstant) -> &[u32] {
  let bytes = unsafe {
    let size_in_bytes = std::mem::size_of::<PushConstant>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = push_constants as *const PushConstant as *const u32;
    std::slice::from_raw_parts(ptr, size_in_u32)
  };

  return bytes;
}

impl Default for TrianglesComponent {
  /// Load in shaders upon creation.

  fn default() -> Self {
    // Specify virtual shaders to use for rendering
    let triangle_vertex = VirtualShader::Source {
      source: include_str!("../assets/shaders/triangles.vert").to_string(),
      kind: ShaderKind::Vertex,
      name: String::from("triangles"),
      entry_point: String::from("main"),
    };

    let triangle_fragment = VirtualShader::Source {
      source: include_str!("../assets/shaders/triangles.frag").to_string(),
      kind: ShaderKind::Fragment,
      name: String::from("triangles"),
      entry_point: String::from("main"),
    };

    // Create a shader builder to compile the shaders.
    let mut builder = ShaderBuilder::new();
    let vs = builder.build(triangle_vertex);
    let fs = builder.build(triangle_fragment);

    return TrianglesComponent {
      vertex_shader: vs,
      triangle_vertex: fs,
      render_pass: None,
      render_pipeline: None,
      width: 800,
      height: 600,
      animation_scalar: 0.0,
      position: (0.0, 0.0),
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Multiple Triangles Demo")
    .with_renderer_configured_as(move |render_context_builder| {
      return render_context_builder.with_render_timeout(1_000_000_000);
    })
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(800, 600)
        .with_name("Triangles");
    })
    .with_component(move |runtime, triangles: TrianglesComponent| {
      return (runtime, triangles);
    })
    .build();

  start_runtime(runtime);
}
