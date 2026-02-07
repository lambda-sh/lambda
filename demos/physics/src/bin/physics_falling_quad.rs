#![allow(clippy::needless_return)]

//! Demo: Render a falling quad using fixed-timestep stepping.
//!
//! This demo intentionally does not use rigid bodies or colliders. The quad's
//! motion is integrated with simple kinematics using `PhysicsWorld2D` gravity
//! and timestep settings. The physics world is stepped to validate fixed
//! timestep and sub-step behavior for an empty world.

use lambda::{
  component::Component,
  events::{
    EventMask,
    WindowEvent,
  },
  physics::{
    PhysicsWorld2D,
    PhysicsWorld2DBuilder,
  },
  render::{
    bind::{
      BindGroupBuilder,
      BindGroupLayoutBuilder,
      BindingVisibility,
    },
    buffer::{
      Buffer,
      BufferBuilder,
      Properties,
      Usage,
    },
    command::RenderCommand,
    mesh::{
      Mesh,
      MeshBuilder,
    },
    pipeline::{
      CullingMode,
      RenderPipelineBuilder,
    },
    render_pass::RenderPassBuilder,
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    vertex::{
      ColorFormat,
      VertexAttribute,
      VertexBuilder,
      VertexElement,
    },
    viewport::ViewportBuilder,
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

layout (set = 0, binding = 0) uniform QuadGlobals {
  vec4 offset;
} globals;

void main() {
  vec2 translated = vertex_position.xy + globals.offset.xy;
  gl_Position = vec4(translated, vertex_position.z, 1.0);
  frag_color = vertex_color;
}

"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 frag_color;
layout (location = 0) out vec4 fragment_color;

void main() {
  fragment_color = vec4(frag_color, 1.0);
}

"#;

// ---------------------------- UNIFORM STRUCTURE ------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuadGlobalsUniform {
  pub offset: [f32; 4],
}

unsafe impl lambda::pod::PlainOldData for QuadGlobalsUniform {}

// --------------------------------- COMPONENT ---------------------------------

pub struct FallingQuadDemo {
  physics_world: PhysicsWorld2D,
  physics_accumulator_seconds: f32,
  quad_position_y: f32,
  quad_velocity_y: f32,
  floor_y: f32,
  restitution: f32,

  vertex_shader: Shader,
  fragment_shader: Shader,
  mesh: Option<Mesh>,
  render_pipeline_id: Option<ResourceId>,
  render_pass_id: Option<ResourceId>,
  bind_group_id: Option<ResourceId>,
  uniform_buffer: Option<Buffer>,

  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for FallingQuadDemo {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new()
      .with_label("physics-falling-quad-pass")
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
      );

    let quad_half_size = 0.08_f32;
    let vertices = [
      VertexBuilder::new()
        .with_position([-quad_half_size, -quad_half_size, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 0.3, 0.2])
        .build(),
      VertexBuilder::new()
        .with_position([quad_half_size, -quad_half_size, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.2, 1.0, 0.3])
        .build(),
      VertexBuilder::new()
        .with_position([quad_half_size, quad_half_size, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.2, 0.3, 1.0])
        .build(),
      VertexBuilder::new()
        .with_position([-quad_half_size, -quad_half_size, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 0.3, 0.2])
        .build(),
      VertexBuilder::new()
        .with_position([quad_half_size, quad_half_size, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.2, 0.3, 1.0])
        .build(),
      VertexBuilder::new()
        .with_position([-quad_half_size, quad_half_size, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.2])
        .build(),
    ];

    let mut mesh_builder = MeshBuilder::new();
    vertices.iter().for_each(|vertex| {
      mesh_builder.with_vertex(*vertex);
    });

    let mesh = mesh_builder
      .with_attributes(vec![
        VertexAttribute {
          location: 0,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 0,
          },
        },
        VertexAttribute {
          location: 1,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 12,
          },
        },
        VertexAttribute {
          location: 2,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 24,
          },
        },
      ])
      .build();

    let layout = BindGroupLayoutBuilder::new()
      .with_uniform(0, BindingVisibility::Vertex)
      .build(render_context.gpu());

    let initial_uniform = QuadGlobalsUniform {
      offset: [0.0, self.quad_position_y, 0.0, 0.0],
    };

    let uniform_buffer = BufferBuilder::new()
      .with_length(std::mem::size_of::<QuadGlobalsUniform>())
      .with_usage(Usage::UNIFORM)
      .with_properties(Properties::CPU_VISIBLE)
      .with_label("quad-globals")
      .build(render_context.gpu(), vec![initial_uniform])
      .map_err(|error| error.to_string())?;

    let bind_group = BindGroupBuilder::new()
      .with_layout(&layout)
      .with_uniform(0, &uniform_buffer, 0, None)
      .build(render_context.gpu());

    let pipeline = RenderPipelineBuilder::new()
      .with_label("physics-falling-quad-pipeline")
      .with_culling(CullingMode::None)
      .with_layouts(&[&layout])
      .with_buffer(
        BufferBuilder::build_from_mesh(&mesh, render_context.gpu())
          .map_err(|error| error.to_string())?,
        mesh.attributes().to_vec(),
      )
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &render_pass,
        &self.vertex_shader,
        Some(&self.fragment_shader),
      );

    self.render_pass_id = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline_id = Some(render_context.attach_pipeline(pipeline));
    self.mesh = Some(mesh);
    self.uniform_buffer = Some(uniform_buffer);
    self.bind_group_id = Some(render_context.attach_bind_group(bind_group));

    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    match event {
      WindowEvent::Resize { width, height } => {
        self.width = *width;
        self.height = *height;
      }
      WindowEvent::Close => {}
    }

    return Ok(());
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.physics_accumulator_seconds += last_frame.as_secs_f32();

    let timestep_seconds = self.physics_world.timestep_seconds();
    let gravity = self.physics_world.gravity();

    while self.physics_accumulator_seconds >= timestep_seconds {
      self.physics_world.step();

      self.quad_velocity_y += gravity[1] * timestep_seconds;
      self.quad_position_y += self.quad_velocity_y * timestep_seconds;

      if self.quad_position_y < self.floor_y {
        self.quad_position_y = self.floor_y;
        self.quad_velocity_y = -self.quad_velocity_y * self.restitution;
      }

      self.physics_accumulator_seconds -= timestep_seconds;
    }

    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport = ViewportBuilder::new().build(self.width, self.height);

    if let Some(ref uniform_buffer) = self.uniform_buffer {
      let value = QuadGlobalsUniform {
        offset: [0.0, self.quad_position_y, 0.0, 0.0],
      };
      uniform_buffer.write_value(render_context.gpu(), 0, &value);
    }

    let render_pass = self.render_pass_id.expect("render pass missing");
    let render_pipeline =
      self.render_pipeline_id.expect("render pipeline missing");
    let bind_group = self.bind_group_id.expect("bind group missing");

    return vec![
      RenderCommand::BeginRenderPass {
        render_pass,
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: render_pipeline,
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group: bind_group,
        dynamic_offsets: Vec::new(),
      },
      RenderCommand::BindVertexBuffer {
        pipeline: render_pipeline,
        buffer: 0,
      },
      RenderCommand::Draw {
        vertices: 0..self.mesh.as_ref().unwrap().vertices().len() as u32,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for FallingQuadDemo {
  fn default() -> Self {
    let physics_world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -1.5)
      .build()
      .expect("Failed to create PhysicsWorld2D");

    let mut shader_builder = ShaderBuilder::new();
    let vertex_shader = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "physics-falling-quad".to_string(),
    });
    let fragment_shader = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "physics-falling-quad".to_string(),
    });

    return Self {
      physics_world,
      physics_accumulator_seconds: 0.0,
      quad_position_y: 0.8,
      quad_velocity_y: 0.0,
      floor_y: -0.8,
      restitution: 0.6,

      vertex_shader,
      fragment_shader,
      mesh: None,
      render_pipeline_id: None,
      render_pass_id: None,
      bind_group_id: None,
      uniform_buffer: None,
      width: 1200,
      height: 600,
    };
  }
}

fn main() {
  let runtime =
    ApplicationRuntimeBuilder::new("Physics: Falling Quad (Kinematic)")
      .with_renderer_configured_as(move |render_context_builder| {
        return render_context_builder.with_render_timeout(1_000_000_000);
      })
      .with_window_configured_as(move |window_builder| {
        return window_builder
          .with_dimensions(1200, 600)
          .with_name("Physics Falling Quad");
      })
      .with_component(move |runtime, demo: FallingQuadDemo| {
        return (runtime, demo);
      })
      .build();

  start_runtime(runtime);
}
