#![allow(clippy::needless_return)]

//! Demo: Render several 2D rigid bodies and step the simulation.
//!
//! This demo showcases `RigidBody2D` usage without collision shapes:
//! - Two dynamic bodies fall under gravity and respond to forces/impulses.
//! - One kinematic body is moved by user-provided velocity and rotation.
//! - One static body stays fixed (visual reference).
//!
//! Controls:
//! - Space: apply an upward impulse to both dynamic bodies.

use lambda::{
  component::Component,
  events::{
    EventMask,
    Key,
    VirtualKey,
    WindowEvent,
  },
  physics::{
    PhysicsWorld2D,
    PhysicsWorld2DBuilder,
    RigidBody2D,
    RigidBody2DBuilder,
    RigidBodyType,
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
  vec4 offset_rotation;
  vec4 tint;
} globals;

void main() {
  float radians = globals.offset_rotation.z;
  float cosine = cos(radians);
  float sine = sin(radians);

  mat2 rotation = mat2(
    cosine, -sine,
    sine, cosine
  );

  vec2 rotated = rotation * vertex_position.xy;
  vec2 translated = rotated + globals.offset_rotation.xy;

  gl_Position = vec4(translated, vertex_position.z, 1.0);
  frag_color = vertex_color * globals.tint.xyz;
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
  pub offset_rotation: [f32; 4],
  pub tint: [f32; 4],
}

unsafe impl lambda::pod::PlainOldData for QuadGlobalsUniform {}

// -------------------------------- DEMO TYPES ---------------------------------

struct RenderBody {
  body: RigidBody2D,
  tint: [f32; 4],
  uniform_buffer: Buffer,
  bind_group_id: ResourceId,
}

pub struct RigidBodies2DDemo {
  physics_world: PhysicsWorld2D,
  physics_accumulator_seconds: f32,
  pending_impulse: bool,

  dynamic_light_body: RigidBody2D,
  dynamic_heavy_body: RigidBody2D,
  kinematic_body: RigidBody2D,
  static_body: RigidBody2D,

  floor_y: f32,
  ceiling_y: f32,
  left_wall_x: f32,
  right_wall_x: f32,
  restitution: f32,
  wind_force_x_newtons: f32,
  kinematic_rotation_radians: f32,

  vertex_shader: Shader,
  fragment_shader: Shader,
  mesh: Option<Mesh>,
  render_pipeline_id: Option<ResourceId>,
  render_pass_id: Option<ResourceId>,
  bodies: Vec<RenderBody>,

  width: u32,
  height: u32,
}

impl RigidBodies2DDemo {
  fn apply_wall_bounce(&mut self, body: RigidBody2D) -> Result<(), String> {
    let position = body
      .position(&self.physics_world)
      .map_err(|error| error.to_string())?;

    if position[0] <= self.right_wall_x && position[0] >= self.left_wall_x {
      return Ok(());
    }

    let velocity = body
      .velocity(&self.physics_world)
      .map_err(|error| error.to_string())?;

    if position[0] > self.right_wall_x {
      body
        .set_position(&mut self.physics_world, self.right_wall_x, position[1])
        .map_err(|error| error.to_string())?;
      body
        .set_velocity(
          &mut self.physics_world,
          -velocity[0].abs() * self.restitution,
          velocity[1],
        )
        .map_err(|error| error.to_string())?;
    }

    if position[0] < self.left_wall_x {
      body
        .set_position(&mut self.physics_world, self.left_wall_x, position[1])
        .map_err(|error| error.to_string())?;
      body
        .set_velocity(
          &mut self.physics_world,
          velocity[0].abs() * self.restitution,
          velocity[1],
        )
        .map_err(|error| error.to_string())?;
    }

    return Ok(());
  }

  fn apply_floor_bounce(&mut self, body: RigidBody2D) -> Result<(), String> {
    let position = body
      .position(&self.physics_world)
      .map_err(|error| error.to_string())?;

    if position[1] >= self.floor_y {
      return Ok(());
    }

    let velocity = body
      .velocity(&self.physics_world)
      .map_err(|error| error.to_string())?;

    body
      .set_position(&mut self.physics_world, position[0], self.floor_y)
      .map_err(|error| error.to_string())?;

    body
      .set_velocity(
        &mut self.physics_world,
        velocity[0],
        -velocity[1] * self.restitution,
      )
      .map_err(|error| error.to_string())?;

    return Ok(());
  }

  fn apply_ceiling_bounce(&mut self, body: RigidBody2D) -> Result<(), String> {
    let position = body
      .position(&self.physics_world)
      .map_err(|error| error.to_string())?;

    if position[1] <= self.ceiling_y {
      return Ok(());
    }

    let velocity = body
      .velocity(&self.physics_world)
      .map_err(|error| error.to_string())?;

    body
      .set_position(&mut self.physics_world, position[0], self.ceiling_y)
      .map_err(|error| error.to_string())?;

    body
      .set_velocity(
        &mut self.physics_world,
        velocity[0],
        -velocity[1].abs() * self.restitution,
      )
      .map_err(|error| error.to_string())?;

    return Ok(());
  }

  fn clamp_kinematic_within_walls(&mut self) -> Result<(), String> {
    let position = self
      .kinematic_body
      .position(&self.physics_world)
      .map_err(|error| error.to_string())?;

    let velocity = self
      .kinematic_body
      .velocity(&self.physics_world)
      .map_err(|error| error.to_string())?;

    if position[0] > self.right_wall_x {
      self
        .kinematic_body
        .set_position(&mut self.physics_world, self.right_wall_x, position[1])
        .map_err(|error| error.to_string())?;
      self
        .kinematic_body
        .set_velocity(&mut self.physics_world, -velocity[0].abs(), velocity[1])
        .map_err(|error| error.to_string())?;
    }

    if position[0] < self.left_wall_x {
      self
        .kinematic_body
        .set_position(&mut self.physics_world, self.left_wall_x, position[1])
        .map_err(|error| error.to_string())?;
      self
        .kinematic_body
        .set_velocity(&mut self.physics_world, velocity[0].abs(), velocity[1])
        .map_err(|error| error.to_string())?;
    }

    return Ok(());
  }
}

// --------------------------------- COMPONENT ---------------------------------

impl Component<ComponentResult, String> for RigidBodies2DDemo {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new()
      .with_label("physics-rigid-bodies-2d-pass")
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
      );

    let quad_half_size = 0.07_f32;
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

    let mut bodies = Vec::new();
    let demo_bodies = [
      (self.dynamic_light_body, [0.9, 0.35, 0.25, 1.0]),
      (self.dynamic_heavy_body, [0.25, 0.6, 0.95, 1.0]),
      (self.kinematic_body, [0.25, 0.95, 0.45, 1.0]),
      (self.static_body, [0.7, 0.7, 0.7, 1.0]),
    ];

    for (body, tint) in demo_bodies {
      let position = body
        .position(&self.physics_world)
        .map_err(|error| error.to_string())?;
      let rotation = body
        .rotation(&self.physics_world)
        .map_err(|error| error.to_string())?;

      let initial_uniform = QuadGlobalsUniform {
        offset_rotation: [position[0], position[1], rotation, 0.0],
        tint,
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

      bodies.push(RenderBody {
        body,
        tint,
        uniform_buffer,
        bind_group_id: render_context.attach_bind_group(bind_group),
      });
    }

    let pipeline = RenderPipelineBuilder::new()
      .with_label("physics-rigid-bodies-2d-pipeline")
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
    self.bodies = bodies;

    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW | EventMask::KEYBOARD;
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

  fn on_keyboard_event(&mut self, event: &Key) -> Result<(), String> {
    if let Key::Pressed { virtual_key, .. } = event {
      if virtual_key == &Some(VirtualKey::Space) {
        self.pending_impulse = true;
      }
    }

    return Ok(());
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.physics_accumulator_seconds += last_frame.as_secs_f32();

    let timestep_seconds = self.physics_world.timestep_seconds();

    while self.physics_accumulator_seconds >= timestep_seconds {
      self
        .dynamic_light_body
        .apply_force(&mut self.physics_world, self.wind_force_x_newtons, 0.0)
        .map_err(|error| error.to_string())?;
      self
        .dynamic_heavy_body
        .apply_force(&mut self.physics_world, self.wind_force_x_newtons, 0.0)
        .map_err(|error| error.to_string())?;

      if self.pending_impulse {
        let impulse_y_newton_seconds = 1.6;
        self
          .dynamic_light_body
          .apply_impulse(&mut self.physics_world, 0.0, impulse_y_newton_seconds)
          .map_err(|error| error.to_string())?;
        self
          .dynamic_heavy_body
          .apply_impulse(&mut self.physics_world, 0.0, impulse_y_newton_seconds)
          .map_err(|error| error.to_string())?;
        self.pending_impulse = false;
      }

      self.kinematic_rotation_radians += timestep_seconds * 1.1;
      self
        .kinematic_body
        .set_rotation(&mut self.physics_world, self.kinematic_rotation_radians)
        .map_err(|error| error.to_string())?;

      self.physics_world.step();

      self.apply_floor_bounce(self.dynamic_light_body)?;
      self.apply_floor_bounce(self.dynamic_heavy_body)?;
      self.apply_ceiling_bounce(self.dynamic_light_body)?;
      self.apply_ceiling_bounce(self.dynamic_heavy_body)?;
      self.apply_wall_bounce(self.dynamic_light_body)?;
      self.apply_wall_bounce(self.dynamic_heavy_body)?;
      self.clamp_kinematic_within_walls()?;

      self.physics_accumulator_seconds -= timestep_seconds;
    }

    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport = ViewportBuilder::new().build(self.width, self.height);

    for body in self.bodies.iter() {
      let position = body
        .body
        .position(&self.physics_world)
        .expect("RigidBody2D position failed");
      let rotation = body
        .body
        .rotation(&self.physics_world)
        .expect("RigidBody2D rotation failed");

      let value = QuadGlobalsUniform {
        offset_rotation: [position[0], position[1], rotation, 0.0],
        tint: body.tint,
      };

      body
        .uniform_buffer
        .write_value(render_context.gpu(), 0, &value);
    }

    let render_pass = self.render_pass_id.expect("render pass missing");
    let render_pipeline =
      self.render_pipeline_id.expect("render pipeline missing");

    let mut commands = vec![
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
      RenderCommand::BindVertexBuffer {
        pipeline: render_pipeline,
        buffer: 0,
      },
    ];

    for body in self.bodies.iter() {
      commands.push(RenderCommand::SetBindGroup {
        set: 0,
        group: body.bind_group_id,
        dynamic_offsets: Vec::new(),
      });
      commands.push(RenderCommand::Draw {
        vertices: 0..self.mesh.as_ref().unwrap().vertices().len() as u32,
        instances: 0..1,
      });
    }

    commands.push(RenderCommand::EndRenderPass);

    return commands;
  }
}

impl Default for RigidBodies2DDemo {
  fn default() -> Self {
    let mut physics_world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -1.6)
      .build()
      .expect("Failed to create PhysicsWorld2D");

    let dynamic_light_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.35, 0.75)
      .with_dynamic_mass_kg(0.5)
      .build(&mut physics_world)
      .expect("Failed to create dynamic body (light)");

    let dynamic_heavy_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.35, 0.75)
      .with_dynamic_mass_kg(2.0)
      .build(&mut physics_world)
      .expect("Failed to create dynamic body (heavy)");

    let kinematic_body = RigidBody2DBuilder::new(RigidBodyType::Kinematic)
      .with_position(0.0, 0.15)
      .with_velocity(0.6, 0.0)
      .build(&mut physics_world)
      .expect("Failed to create kinematic body");

    let static_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.0, -0.75)
      .build(&mut physics_world)
      .expect("Failed to create static body");

    let mut shader_builder = ShaderBuilder::new();
    let vertex_shader = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "physics-rigid-bodies-2d".to_string(),
    });
    let fragment_shader = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "physics-rigid-bodies-2d".to_string(),
    });

    return Self {
      physics_world,
      physics_accumulator_seconds: 0.0,
      pending_impulse: false,

      dynamic_light_body,
      dynamic_heavy_body,
      kinematic_body,
      static_body,

      floor_y: -0.75,
      ceiling_y: 0.9,
      left_wall_x: -0.85,
      right_wall_x: 0.85,
      restitution: 0.7,
      wind_force_x_newtons: 0.9,
      kinematic_rotation_radians: 0.0,

      vertex_shader,
      fragment_shader,
      mesh: None,
      render_pipeline_id: None,
      render_pass_id: None,
      bodies: Vec::new(),

      width: 1200,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Physics: 2D Rigid Bodies")
    .with_renderer_configured_as(move |render_context_builder| {
      return render_context_builder.with_render_timeout(1_000_000_000);
    })
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("Physics: 2D Rigid Bodies");
    })
    .with_component(move |runtime, demo: RigidBodies2DDemo| {
      return (runtime, demo);
    })
    .build();

  start_runtime(runtime);
}
