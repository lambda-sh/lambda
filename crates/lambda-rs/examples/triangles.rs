#![allow(clippy::needless_return)]
use lambda::{
  component::Component,
  events::{
    EventMask,
    Key,
    VirtualKey,
    WindowEvent,
  },
  render::{
    command::RenderCommand,
    pipeline::{
      self,
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
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
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

impl Component<ComponentResult, String> for TrianglesComponent {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = render_pass::RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    let immediate_data_size = std::mem::size_of::<ImmediateData>() as u32;
    let pipeline = pipeline::RenderPipelineBuilder::new()
      .with_culling(pipeline::CullingMode::None)
      .with_immediate_data(immediate_data_size)
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &render_pass,
        &self.vertex_shader,
        Some(&self.triangle_vertex),
      );

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));

    logging::info!("Attached the DemoComponent.");
    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    let (x, y) = self.position;

    let triangle_data = &[
      ImmediateData {
        color: [
          1.0,
          1.0 * self.animation_scalar,
          0.5 * self.animation_scalar,
          1.0,
        ],
        pos: [x, y],
        scale: [0.3, 0.3],
      },
      ImmediateData {
        color: [0.0, 1.0, 0.0, 1.0],
        pos: [0.5, 0.0],
        scale: [0.4, 0.4],
      },
      ImmediateData {
        color: [0.0, 0.0, 1.0, 1.0],
        pos: [0.25, 0.5],
        scale: [0.5, 0.5],
      },
      ImmediateData {
        color: [1.0, 1.0, 1.0, 1.0],
        pos: [0.0, 0.0],
        scale: [0.5, 0.5],
      },
    ];

    let render_pipeline = self
      .render_pipeline
      .expect("No render pipeline actively set for rendering.");

    // All state setting must be inside the render pass
    let mut commands = vec![RenderCommand::BeginRenderPass {
      render_pass: self
        .render_pass
        .expect("Cannot begin the render pass when it doesn't exist."),
      viewport: viewport.clone(),
    }];

    commands.push(RenderCommand::SetPipeline {
      pipeline: render_pipeline,
    });
    commands.push(RenderCommand::SetViewports {
      start_at: 0,
      viewports: vec![viewport.clone()],
    });
    commands.push(RenderCommand::SetScissors {
      start_at: 0,
      viewports: vec![viewport.clone()],
    });

    // Upload triangle data into the the GPU at the vertex stage of the pipeline
    // before requesting to draw each triangle.
    for triangle in triangle_data {
      commands.push(RenderCommand::Immediates {
        pipeline: render_pipeline,
        offset: 0,
        bytes: Vec::from(immediate_data_to_bytes(triangle)),
      });
      commands.push(RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      });
    }

    commands.push(RenderCommand::EndRenderPass);

    return commands;
  }

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW | EventMask::KEYBOARD;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    match event {
      WindowEvent::Resize { width, height } => {
        logging::info!("Window resized to {}x{}", width, height);
        self.width = *width;
        self.height = *height;
      }
      WindowEvent::Close => {
        logging::info!("Window closed");
      }
    }
    return Ok(());
  }

  fn on_keyboard_event(&mut self, event: &Key) -> Result<(), String> {
    if let Key::Pressed {
      scan_code: _,
      virtual_key,
    } = event
    {
      match virtual_key {
        Some(VirtualKey::KeyW) => {
          self.position.1 -= 0.01;
        }
        Some(VirtualKey::KeyS) => {
          self.position.1 += 0.01;
        }
        Some(VirtualKey::KeyA) => {
          self.position.0 -= 0.01;
        }
        Some(VirtualKey::KeyD) => {
          self.position.0 += 0.01;
        }
        _ => {}
      }
    }
    return Ok(());
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    if last_frame.as_millis() > 20 {
      logging::warn!("Last frame took {}ms", last_frame.as_millis());
    }
    return Ok(ComponentResult::Success);
  }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ImmediateData {
  color: [f32; 4],
  pos: [f32; 2],
  scale: [f32; 2],
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
