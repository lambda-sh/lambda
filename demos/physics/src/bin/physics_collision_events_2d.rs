#![allow(clippy::needless_return)]

//! Demo: Render a 2D collision pair and log collision events.
//!
//! This demo keeps the scene intentionally small so the collision event stream
//! is easy to inspect:
//! - A dynamic ball falls onto a static floor.
//! - `CollisionEventKind::Started` is logged when contact begins.
//! - `CollisionEventKind::Ended` is logged after Space launches the ball away.
//! - The ball tint switches while floor contact is active.
//!
//! Controls:
//! - Space: launch the ball upward after it has settled on the floor

use std::ops::Range;

use lambda::{
  component::Component,
  events::{
    EventMask,
    Key,
    VirtualKey,
    WindowEvent,
  },
  physics::{
    Collider2DBuilder,
    CollisionEvent,
    CollisionEventKind,
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

const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 600;

const FLOOR_HALF_WIDTH: f32 = 0.88;
const FLOOR_HALF_HEIGHT: f32 = 0.05;
const FLOOR_Y: f32 = -0.82;

const BALL_RADIUS: f32 = 0.08;
const BALL_START_Y: f32 = 0.42;
const BALL_LAUNCH_IMPULSE_Y: f32 = 1.45;

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

layout (set = 0, binding = 0) uniform ContactDemoGlobals {
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ContactDemoUniform {
  offset_rotation: [f32; 4],
  tint: [f32; 4],
}

unsafe impl lambda::pod::PlainOldData for ContactDemoUniform {}

struct RenderBody {
  body: RigidBody2D,
  vertices: Range<u32>,
  tint_idle: [f32; 4],
  tint_contact: [f32; 4],
  highlights_contact: bool,
  uniform_buffer: Buffer,
  bind_group_id: ResourceId,
}

pub struct CollisionEvents2DDemo {
  physics_world: PhysicsWorld2D,
  physics_accumulator_seconds: f32,
  pending_launch_impulse: bool,

  ball_body: RigidBody2D,
  floor_body: RigidBody2D,
  ball_contact_active: bool,

  vertex_shader: Shader,
  fragment_shader: Shader,
  mesh: Option<Mesh>,
  render_pipeline_id: Option<ResourceId>,
  render_pass_id: Option<ResourceId>,
  bodies: Vec<RenderBody>,

  width: u32,
  height: u32,
}

impl CollisionEvents2DDemo {
  fn push_vertex(mesh_builder: MeshBuilder, x: f32, y: f32) -> MeshBuilder {
    return mesh_builder.with_vertex(
      VertexBuilder::new()
        .with_position([x, y, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 1.0])
        .build(),
    );
  }

  fn append_rectangle(
    mesh_builder: MeshBuilder,
    vertex_count: &mut u32,
    half_width: f32,
    half_height: f32,
  ) -> (MeshBuilder, Range<u32>) {
    let start = *vertex_count;

    let left = -half_width;
    let right = half_width;
    let bottom = -half_height;
    let top = half_height;

    let mesh_builder = Self::push_vertex(mesh_builder, left, bottom);
    let mesh_builder = Self::push_vertex(mesh_builder, right, bottom);
    let mesh_builder = Self::push_vertex(mesh_builder, right, top);

    let mesh_builder = Self::push_vertex(mesh_builder, left, bottom);
    let mesh_builder = Self::push_vertex(mesh_builder, right, top);
    let mesh_builder = Self::push_vertex(mesh_builder, left, top);

    *vertex_count += 6;
    let end = *vertex_count;
    return (mesh_builder, start..end);
  }

  fn append_circle(
    mesh_builder: MeshBuilder,
    vertex_count: &mut u32,
    radius: f32,
    segments: u32,
  ) -> (MeshBuilder, Range<u32>) {
    let start = *vertex_count;
    let mut mesh_builder = mesh_builder;

    for index in 0..segments {
      let t0 = index as f32 / segments as f32;
      let t1 = (index + 1) as f32 / segments as f32;

      let angle0 = t0 * std::f32::consts::TAU;
      let angle1 = t1 * std::f32::consts::TAU;

      let x0 = angle0.cos() * radius;
      let y0 = angle0.sin() * radius;
      let x1 = angle1.cos() * radius;
      let y1 = angle1.sin() * radius;

      mesh_builder = Self::push_vertex(mesh_builder, 0.0, 0.0);
      mesh_builder = Self::push_vertex(mesh_builder, x0, y0);
      mesh_builder = Self::push_vertex(mesh_builder, x1, y1);
    }

    *vertex_count += 3 * segments;
    let end = *vertex_count;
    return (mesh_builder, start..end);
  }

  fn is_ball_floor_event(&self, event: CollisionEvent) -> bool {
    let is_direct_pair =
      event.body_a == self.ball_body && event.body_b == self.floor_body;
    let is_swapped_pair =
      event.body_a == self.floor_body && event.body_b == self.ball_body;
    return is_direct_pair || is_swapped_pair;
  }

  fn log_ball_floor_event(&mut self, event: CollisionEvent) {
    match event.kind {
      CollisionEventKind::Started => {
        self.ball_contact_active = true;

        match (event.contact_point, event.normal, event.penetration) {
          (Some(point), Some(normal), Some(penetration)) => {
            println!(
              "Collision Started: point=({:.3}, {:.3}) normal=({:.3}, {:.3}) penetration={:.4}",
              point[0],
              point[1],
              normal[0],
              normal[1],
              penetration,
            );
          }
          _ => {
            println!("Collision Started: contact data unavailable");
          }
        }
      }
      CollisionEventKind::Ended => {
        self.ball_contact_active = false;
        println!("Collision Ended: ball left the floor");
      }
    }

    return;
  }
}

impl Component<ComponentResult, String> for CollisionEvents2DDemo {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    println!(
      "Controls: wait for the ball to settle on the floor, then press Space to launch it."
    );

    let render_pass = RenderPassBuilder::new()
      .with_label("physics-collision-events-2d-pass")
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
      );

    let layout = BindGroupLayoutBuilder::new()
      .with_uniform(0, BindingVisibility::Vertex)
      .build(render_context.gpu());

    let attributes = vec![
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
    ];

    let mut mesh_builder =
      MeshBuilder::new().with_attributes(attributes.clone());
    let mut vertex_count = 0_u32;

    let (updated_mesh_builder, floor_vertices) = Self::append_rectangle(
      mesh_builder,
      &mut vertex_count,
      FLOOR_HALF_WIDTH,
      FLOOR_HALF_HEIGHT,
    );
    mesh_builder = updated_mesh_builder;

    let (updated_mesh_builder, ball_vertices) =
      Self::append_circle(mesh_builder, &mut vertex_count, BALL_RADIUS, 32);
    mesh_builder = updated_mesh_builder;

    let mesh = mesh_builder.build();

    let render_bodies = [
      (
        self.floor_body,
        floor_vertices,
        [0.22, 0.22, 0.24, 1.0],
        [0.22, 0.22, 0.24, 1.0],
        false,
      ),
      (
        self.ball_body,
        ball_vertices,
        [0.22, 0.55, 0.95, 1.0],
        [0.95, 0.28, 0.22, 1.0],
        true,
      ),
    ];

    let mut bodies = Vec::with_capacity(render_bodies.len());

    for (body, vertices, tint_idle, tint_contact, highlights_contact) in
      render_bodies
    {
      let position = body
        .position(&self.physics_world)
        .map_err(|error| error.to_string())?;
      let rotation = body
        .rotation(&self.physics_world)
        .map_err(|error| error.to_string())?;

      let initial_uniform = ContactDemoUniform {
        offset_rotation: [position[0], position[1], rotation, 0.0],
        tint: tint_idle,
      };

      let uniform_buffer = BufferBuilder::new()
        .with_length(std::mem::size_of::<ContactDemoUniform>())
        .with_usage(Usage::UNIFORM)
        .with_properties(Properties::CPU_VISIBLE)
        .with_label("collision-events-demo-globals")
        .build(render_context.gpu(), vec![initial_uniform])
        .map_err(|error| error.to_string())?;

      let bind_group = BindGroupBuilder::new()
        .with_layout(&layout)
        .with_uniform(0, &uniform_buffer, 0, None)
        .build(render_context.gpu());

      bodies.push(RenderBody {
        body,
        vertices,
        tint_idle,
        tint_contact,
        highlights_contact,
        uniform_buffer,
        bind_group_id: render_context.attach_bind_group(bind_group),
      });
    }

    let pipeline = RenderPipelineBuilder::new()
      .with_label("physics-collision-events-2d-pipeline")
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
    let Key::Pressed { virtual_key, .. } = event else {
      return Ok(());
    };

    if virtual_key != &Some(VirtualKey::Space) {
      return Ok(());
    }

    if !self.ball_contact_active {
      println!("Space ignored: wait until the ball is resting on the floor");
      return Ok(());
    }

    self.pending_launch_impulse = true;
    return Ok(());
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.physics_accumulator_seconds += last_frame.as_secs_f32();

    let timestep_seconds = self.physics_world.timestep_seconds();

    while self.physics_accumulator_seconds >= timestep_seconds {
      if self.pending_launch_impulse {
        self
          .ball_body
          .set_velocity(&mut self.physics_world, 0.0, 0.0)
          .map_err(|error| error.to_string())?;
        self
          .ball_body
          .apply_impulse(&mut self.physics_world, 0.0, BALL_LAUNCH_IMPULSE_Y)
          .map_err(|error| error.to_string())?;
        self.pending_launch_impulse = false;
        println!("Launch impulse applied");
      }

      self.physics_world.step();

      for event in self.physics_world.collision_events() {
        if self.is_ball_floor_event(event) {
          self.log_ball_floor_event(event);
        }
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

    for body in self.bodies.iter() {
      let position = body
        .body
        .position(&self.physics_world)
        .expect("RigidBody2D position failed");
      let rotation = body
        .body
        .rotation(&self.physics_world)
        .expect("RigidBody2D rotation failed");

      let tint = if body.highlights_contact && self.ball_contact_active {
        body.tint_contact
      } else {
        body.tint_idle
      };

      let uniform = ContactDemoUniform {
        offset_rotation: [position[0], position[1], rotation, 0.0],
        tint,
      };

      body
        .uniform_buffer
        .write_value(render_context.gpu(), 0, &uniform);
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
        vertices: body.vertices.clone(),
        instances: 0..1,
      });
    }

    commands.push(RenderCommand::EndRenderPass);
    return commands;
  }
}

impl Default for CollisionEvents2DDemo {
  fn default() -> Self {
    let mut physics_world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -3.2)
      .with_substeps(4)
      .build()
      .expect("Failed to create PhysicsWorld2D");

    let floor_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.0, FLOOR_Y)
      .build(&mut physics_world)
      .expect("Failed to create floor body");

    Collider2DBuilder::rectangle(FLOOR_HALF_WIDTH, FLOOR_HALF_HEIGHT)
      .with_density(0.0)
      .with_friction(0.8)
      .with_restitution(0.0)
      .build(&mut physics_world, floor_body)
      .expect("Failed to create floor collider");

    let ball_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.0, BALL_START_Y)
      .build(&mut physics_world)
      .expect("Failed to create ball body");

    Collider2DBuilder::circle(BALL_RADIUS)
      .with_density(100.0)
      .with_friction(0.45)
      .with_restitution(0.0)
      .build(&mut physics_world, ball_body)
      .expect("Failed to create ball collider");

    let mut shader_builder = ShaderBuilder::new();
    let vertex_shader = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "physics-collision-events-2d".to_string(),
    });
    let fragment_shader = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "physics-collision-events-2d".to_string(),
    });

    return Self {
      physics_world,
      physics_accumulator_seconds: 0.0,
      pending_launch_impulse: false,

      ball_body,
      floor_body,
      ball_contact_active: false,

      vertex_shader,
      fragment_shader,
      mesh: None,
      render_pipeline_id: None,
      render_pass_id: None,
      bodies: Vec::new(),

      width: WINDOW_WIDTH,
      height: WINDOW_HEIGHT,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Physics: 2D Collision Events")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
        .with_name("Physics: 2D Collision Events");
    })
    .with_component(move |runtime, demo: CollisionEvents2DDemo| {
      return (runtime, demo);
    })
    .build();

  start_runtime(runtime);
}
