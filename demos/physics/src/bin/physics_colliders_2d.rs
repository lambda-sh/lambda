#![allow(clippy::needless_return)]

//! Demo: Render several 2D colliders and step the simulation.
//!
//! This demo showcases `Collider2DBuilder` usage for common primitives:
//! - Circle colliders (restitution and density)
//! - Rectangle/box colliders (including local rotation)
//! - Capsule colliders (character-like shape)
//! - Convex polygon colliders
//! - Multiple colliders attached to one body (compound)
//! - Friction and restitution affecting collision response
//!
//! Controls:
//! - Space: apply the same impulse to the density demo bodies

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
    ColliderMaterial2D,
    ColliderShape2D,
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

layout (set = 0, binding = 0) uniform ColliderGlobals {
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
pub struct ColliderGlobalsUniform {
  pub offset_rotation: [f32; 4],
  pub tint: [f32; 4],
}

unsafe impl lambda::pod::PlainOldData for ColliderGlobalsUniform {}

// -------------------------------- DEMO TYPES ---------------------------------

#[derive(Debug, Clone)]
struct ColliderRenderInit {
  body: RigidBody2D,
  shape: ColliderShape2D,
  local_offset: [f32; 2],
  local_rotation: f32,
  tint: [f32; 4],
}

struct RenderCollider {
  init: ColliderRenderInit,
  vertices: Range<u32>,
  uniform_buffer: Buffer,
  bind_group_id: ResourceId,
}

pub struct Colliders2DDemo {
  physics_world: PhysicsWorld2D,
  physics_accumulator_seconds: f32,

  pending_impulse: bool,
  impulse_cooldown_remaining_seconds: f32,
  density_light_body: RigidBody2D,
  density_heavy_body: RigidBody2D,

  vertex_shader: Shader,
  fragment_shader: Shader,
  mesh: Option<Mesh>,
  render_pipeline_id: Option<ResourceId>,
  render_pass_id: Option<ResourceId>,
  colliders: Vec<RenderCollider>,
  collider_inits: Vec<ColliderRenderInit>,

  width: u32,
  height: u32,
}

impl Colliders2DDemo {
  fn push_vertex(
    mesh_builder: &mut MeshBuilder,
    x: f32,
    y: f32,
  ) -> &mut MeshBuilder {
    return mesh_builder.with_vertex(
      VertexBuilder::new()
        .with_position([x, y, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 1.0])
        .build(),
    );
  }

  fn append_rectangle(
    mesh_builder: &mut MeshBuilder,
    vertex_count: &mut u32,
    half_width: f32,
    half_height: f32,
  ) -> Range<u32> {
    let start = *vertex_count;

    let left = -half_width;
    let right = half_width;
    let bottom = -half_height;
    let top = half_height;

    Self::push_vertex(mesh_builder, left, bottom);
    Self::push_vertex(mesh_builder, right, bottom);
    Self::push_vertex(mesh_builder, right, top);

    Self::push_vertex(mesh_builder, left, bottom);
    Self::push_vertex(mesh_builder, right, top);
    Self::push_vertex(mesh_builder, left, top);

    *vertex_count += 6;
    let end = *vertex_count;
    return start..end;
  }

  fn append_convex_polygon(
    mesh_builder: &mut MeshBuilder,
    vertex_count: &mut u32,
    vertices: &[[f32; 2]],
  ) -> Range<u32> {
    let start = *vertex_count;

    let mut centroid = [0.0_f32, 0.0_f32];
    for vertex in vertices.iter() {
      centroid[0] += vertex[0];
      centroid[1] += vertex[1];
    }
    centroid[0] /= vertices.len() as f32;
    centroid[1] /= vertices.len() as f32;

    for index in 0..vertices.len() {
      let a = vertices[index];
      let b = vertices[(index + 1) % vertices.len()];

      Self::push_vertex(mesh_builder, centroid[0], centroid[1]);
      Self::push_vertex(mesh_builder, a[0], a[1]);
      Self::push_vertex(mesh_builder, b[0], b[1]);
    }

    *vertex_count += 3 * vertices.len() as u32;
    let end = *vertex_count;
    return start..end;
  }

  fn append_circle(
    mesh_builder: &mut MeshBuilder,
    vertex_count: &mut u32,
    radius: f32,
    segments: u32,
  ) -> Range<u32> {
    let start = *vertex_count;

    for index in 0..segments {
      let t0 = index as f32 / segments as f32;
      let t1 = (index + 1) as f32 / segments as f32;

      let angle0 = t0 * std::f32::consts::TAU;
      let angle1 = t1 * std::f32::consts::TAU;

      let x0 = angle0.cos() * radius;
      let y0 = angle0.sin() * radius;
      let x1 = angle1.cos() * radius;
      let y1 = angle1.sin() * radius;

      Self::push_vertex(mesh_builder, 0.0, 0.0);
      Self::push_vertex(mesh_builder, x0, y0);
      Self::push_vertex(mesh_builder, x1, y1);
    }

    *vertex_count += 3 * segments;
    let end = *vertex_count;
    return start..end;
  }

  fn append_capsule(
    mesh_builder: &mut MeshBuilder,
    vertex_count: &mut u32,
    half_height: f32,
    radius: f32,
    segments: u32,
  ) -> Range<u32> {
    let mut vertices = Vec::with_capacity((segments as usize + 1) * 2);

    for index in 0..=segments {
      let t = index as f32 / segments as f32;
      let angle = t * std::f32::consts::PI;
      vertices.push([angle.cos() * radius, angle.sin() * radius + half_height]);
    }

    for index in 0..=segments {
      let t = index as f32 / segments as f32;
      let angle = std::f32::consts::PI + t * std::f32::consts::PI;
      vertices.push([angle.cos() * radius, angle.sin() * radius - half_height]);
    }

    return Self::append_convex_polygon(mesh_builder, vertex_count, &vertices);
  }

  fn append_shape(
    mesh_builder: &mut MeshBuilder,
    vertex_count: &mut u32,
    shape: &ColliderShape2D,
  ) -> Range<u32> {
    match shape {
      ColliderShape2D::Circle { radius } => {
        return Self::append_circle(mesh_builder, vertex_count, *radius, 32);
      }
      ColliderShape2D::Rectangle {
        half_width,
        half_height,
      } => {
        return Self::append_rectangle(
          mesh_builder,
          vertex_count,
          *half_width,
          *half_height,
        );
      }
      ColliderShape2D::Capsule {
        half_height,
        radius,
      } => {
        return Self::append_capsule(
          mesh_builder,
          vertex_count,
          *half_height,
          *radius,
          16,
        );
      }
      ColliderShape2D::ConvexPolygon { vertices } => {
        return Self::append_convex_polygon(
          mesh_builder,
          vertex_count,
          vertices,
        );
      }
    }
  }

  fn rotate_vector(radians: f32, vector: [f32; 2]) -> [f32; 2] {
    let cosine = radians.cos();
    let sine = radians.sin();
    return [
      cosine * vector[0] - sine * vector[1],
      sine * vector[0] + cosine * vector[1],
    ];
  }
}

impl Component<ComponentResult, String> for Colliders2DDemo {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new()
      .with_label("physics-colliders-2d-pass")
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

    let mut mesh_builder = MeshBuilder::new();
    mesh_builder.with_attributes(attributes.clone());
    let mut vertex_count = 0_u32;

    let mut colliders = Vec::with_capacity(self.collider_inits.len());

    for init in self.collider_inits.iter() {
      let vertices =
        Self::append_shape(&mut mesh_builder, &mut vertex_count, &init.shape);

      let initial_uniform = ColliderGlobalsUniform {
        offset_rotation: [0.0, 0.0, 0.0, 0.0],
        tint: init.tint,
      };

      let uniform_buffer = BufferBuilder::new()
        .with_length(std::mem::size_of::<ColliderGlobalsUniform>())
        .with_usage(Usage::UNIFORM)
        .with_properties(Properties::CPU_VISIBLE)
        .with_label("collider-globals")
        .build(render_context.gpu(), vec![initial_uniform])
        .map_err(|error| error.to_string())?;

      let bind_group = BindGroupBuilder::new()
        .with_layout(&layout)
        .with_uniform(0, &uniform_buffer, 0, None)
        .build(render_context.gpu());

      colliders.push(RenderCollider {
        init: init.clone(),
        vertices,
        uniform_buffer,
        bind_group_id: render_context.attach_bind_group(bind_group),
      });
    }

    let mesh = mesh_builder.build();

    let pipeline = RenderPipelineBuilder::new()
      .with_label("physics-colliders-2d-pipeline")
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
    self.colliders = colliders;

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

    if self.impulse_cooldown_remaining_seconds > 0.0 {
      return Ok(());
    }

    self.pending_impulse = true;
    self.impulse_cooldown_remaining_seconds = 0.25;
    println!("Space pressed: applying impulse to density demo bodies");

    return Ok(());
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.impulse_cooldown_remaining_seconds =
      (self.impulse_cooldown_remaining_seconds - last_frame.as_secs_f32())
        .max(0.0);

    self.physics_accumulator_seconds += last_frame.as_secs_f32();

    let timestep_seconds = self.physics_world.timestep_seconds();

    while self.physics_accumulator_seconds >= timestep_seconds {
      if self.pending_impulse {
        let impulse_x_newton_seconds = 0.0;
        let impulse_y_newton_seconds = 1.2;

        self
          .density_light_body
          .set_velocity(&mut self.physics_world, 0.0, 0.0)
          .map_err(|error| error.to_string())?;
        self
          .density_heavy_body
          .set_velocity(&mut self.physics_world, 0.0, 0.0)
          .map_err(|error| error.to_string())?;

        self
          .density_light_body
          .apply_impulse(
            &mut self.physics_world,
            impulse_x_newton_seconds,
            impulse_y_newton_seconds,
          )
          .map_err(|error| error.to_string())?;
        self
          .density_heavy_body
          .apply_impulse(
            &mut self.physics_world,
            impulse_x_newton_seconds,
            impulse_y_newton_seconds,
          )
          .map_err(|error| error.to_string())?;

        let velocity_light = self
          .density_light_body
          .velocity(&self.physics_world)
          .map_err(|error| error.to_string())?;
        let velocity_heavy = self
          .density_heavy_body
          .velocity(&self.physics_world)
          .map_err(|error| error.to_string())?;

        println!(
          "Impulse applied. light v=({:.2}, {:.2}) heavy v=({:.2}, {:.2})",
          velocity_light[0],
          velocity_light[1],
          velocity_heavy[0],
          velocity_heavy[1],
        );

        self.pending_impulse = false;
      }

      self.physics_world.step();
      self.physics_accumulator_seconds -= timestep_seconds;
    }

    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport = ViewportBuilder::new().build(self.width, self.height);

    for collider in self.colliders.iter() {
      let body_position = collider
        .init
        .body
        .position(&self.physics_world)
        .expect("RigidBody2D position failed");
      let body_rotation = collider
        .init
        .body
        .rotation(&self.physics_world)
        .expect("RigidBody2D rotation failed");

      // The clip-space coordinate system used by this demo mirrors Rapier's
      // angular sign convention on screen. Negating the sampled physics angle
      // keeps rendered collider orientation aligned with the collision shape
      // that Rapier is simulating.
      let visual_body_rotation = -body_rotation;

      let rotated_offset =
        Self::rotate_vector(visual_body_rotation, collider.init.local_offset);

      let offset = [
        body_position[0] + rotated_offset[0],
        body_position[1] + rotated_offset[1],
      ];

      let rotation = -(body_rotation + collider.init.local_rotation);

      let uniform = ColliderGlobalsUniform {
        offset_rotation: [offset[0], offset[1], rotation, 0.0],
        tint: collider.init.tint,
      };

      collider
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

    for collider in self.colliders.iter() {
      commands.push(RenderCommand::SetBindGroup {
        set: 0,
        group: collider.bind_group_id,
        dynamic_offsets: Vec::new(),
      });
      commands.push(RenderCommand::Draw {
        vertices: collider.vertices.clone(),
        instances: 0..1,
      });
    }

    commands.push(RenderCommand::EndRenderPass);
    return commands;
  }
}

impl Default for Colliders2DDemo {
  fn default() -> Self {
    let mut physics_world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -3.2)
      .with_substeps(4)
      .build()
      .expect("Failed to create PhysicsWorld2D");

    let mut collider_inits = Vec::new();

    let ground_material = ColliderMaterial2D::new(0.0, 0.8, 0.0);
    let ramp_material = ColliderMaterial2D::new(0.0, 1.0, 0.0);

    let ground_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.0, -0.86)
      .build(&mut physics_world)
      .expect("Failed to create ground body");
    Collider2DBuilder::rectangle(0.95, 0.05)
      .with_density(ground_material.density())
      .with_friction(ground_material.friction())
      .with_restitution(ground_material.restitution())
      .build(&mut physics_world, ground_body)
      .expect("Failed to create ground collider");
    collider_inits.push(ColliderRenderInit {
      body: ground_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.95,
        half_height: 0.05,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.25, 0.25, 0.25, 1.0],
    });

    let left_wall_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(-0.98, 0.0)
      .build(&mut physics_world)
      .expect("Failed to create left wall body");
    Collider2DBuilder::rectangle(0.03, 0.95)
      .with_density(0.0)
      .with_friction(0.8)
      .with_restitution(0.0)
      .build(&mut physics_world, left_wall_body)
      .expect("Failed to create left wall collider");
    collider_inits.push(ColliderRenderInit {
      body: left_wall_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.03,
        half_height: 0.95,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.18, 0.18, 0.18, 1.0],
    });

    let right_wall_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.98, 0.0)
      .build(&mut physics_world)
      .expect("Failed to create right wall body");
    Collider2DBuilder::rectangle(0.03, 0.95)
      .with_density(0.0)
      .with_friction(0.8)
      .with_restitution(0.0)
      .build(&mut physics_world, right_wall_body)
      .expect("Failed to create right wall collider");
    collider_inits.push(ColliderRenderInit {
      body: right_wall_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.03,
        half_height: 0.95,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.18, 0.18, 0.18, 1.0],
    });

    let ceiling_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.0, 0.92)
      .build(&mut physics_world)
      .expect("Failed to create ceiling body");
    Collider2DBuilder::rectangle(0.95, 0.03)
      .with_density(0.0)
      .with_friction(0.8)
      .with_restitution(0.0)
      .build(&mut physics_world, ceiling_body)
      .expect("Failed to create ceiling collider");
    collider_inits.push(ColliderRenderInit {
      body: ceiling_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.95,
        half_height: 0.03,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.18, 0.18, 0.18, 1.0],
    });

    // Divider keeps the density demo isolated so Space remains visible.
    let divider_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.32, 0.0)
      .build(&mut physics_world)
      .expect("Failed to create divider body");
    Collider2DBuilder::rectangle(0.02, 0.95)
      .with_density(0.0)
      .with_friction(0.8)
      .with_restitution(0.0)
      .build(&mut physics_world, divider_body)
      .expect("Failed to create divider collider");
    collider_inits.push(ColliderRenderInit {
      body: divider_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.02,
        half_height: 0.95,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.18, 0.18, 0.18, 1.0],
    });

    let ramp_rotation = 0.5_f32;
    let ramp_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(-0.45, -0.25)
      .with_rotation(ramp_rotation)
      .build(&mut physics_world)
      .expect("Failed to create ramp body");
    Collider2DBuilder::rectangle(0.55, 0.03)
      .with_density(ramp_material.density())
      // Keep the ramp somewhat slippery so bodies settle on the floor instead
      // of sticking on the slope for long periods.
      .with_friction(0.4)
      .with_restitution(ramp_material.restitution())
      .build(&mut physics_world, ramp_body)
      .expect("Failed to create ramp collider");
    collider_inits.push(ColliderRenderInit {
      body: ramp_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.55,
        half_height: 0.03,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.3, 0.28, 0.25, 1.0],
    });

    // Restitution demo: bouncy and non-bouncy circles.
    let bouncy_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.75, 0.65)
      .build(&mut physics_world)
      .expect("Failed to create bouncy body");
    Collider2DBuilder::circle(0.06)
      .with_density(110.0)
      .with_friction(0.2)
      .with_restitution(0.95)
      .build(&mut physics_world, bouncy_body)
      .expect("Failed to create bouncy collider");
    collider_inits.push(ColliderRenderInit {
      body: bouncy_body,
      shape: ColliderShape2D::Circle { radius: 0.06 },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [1.0, 0.25, 0.25, 1.0],
    });

    let dull_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.6, 0.65)
      .build(&mut physics_world)
      .expect("Failed to create dull body");
    Collider2DBuilder::circle(0.06)
      .with_density(110.0)
      .with_friction(0.2)
      .with_restitution(0.0)
      .build(&mut physics_world, dull_body)
      .expect("Failed to create dull collider");
    collider_inits.push(ColliderRenderInit {
      body: dull_body,
      shape: ColliderShape2D::Circle { radius: 0.06 },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.65, 0.15, 0.15, 1.0],
    });

    // Friction demo: two boxes on the ramp.
    let slippery_box_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.35, 0.25)
      .build(&mut physics_world)
      .expect("Failed to create slippery box body");
    Collider2DBuilder::rectangle(0.07, 0.07)
      .with_density(100.0)
      .with_friction(0.0)
      .with_restitution(0.0)
      .build(&mut physics_world, slippery_box_body)
      .expect("Failed to create slippery box collider");
    collider_inits.push(ColliderRenderInit {
      body: slippery_box_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.07,
        half_height: 0.07,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.9, 0.9, 0.3, 1.0],
    });

    let grippy_box_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.15, 0.25)
      .build(&mut physics_world)
      .expect("Failed to create grippy box body");
    Collider2DBuilder::rectangle(0.07, 0.07)
      .with_density(100.0)
      .with_friction(1.0)
      .with_restitution(0.0)
      .build(&mut physics_world, grippy_box_body)
      .expect("Failed to create grippy box collider");
    collider_inits.push(ColliderRenderInit {
      body: grippy_box_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.07,
        half_height: 0.07,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.3, 0.9, 0.45, 1.0],
    });

    // Capsule demo: a character-like capsule.
    let capsule_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.25, 0.65)
      .build(&mut physics_world)
      .expect("Failed to create capsule body");
    Collider2DBuilder::capsule(0.09, 0.04)
      .with_density(120.0)
      .with_friction(0.6)
      .with_restitution(0.0)
      .build(&mut physics_world, capsule_body)
      .expect("Failed to create capsule collider");
    collider_inits.push(ColliderRenderInit {
      body: capsule_body,
      shape: ColliderShape2D::Capsule {
        half_height: 0.09,
        radius: 0.04,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.3, 0.55, 0.95, 1.0],
    });

    // Convex polygon demo: a simple triangle.
    let polygon_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.15, 0.72)
      .with_rotation(0.25)
      .build(&mut physics_world)
      .expect("Failed to create polygon body");
    let triangle_vertices = vec![[0.0, 0.1], [-0.09, -0.06], [0.09, -0.06]];
    Collider2DBuilder::polygon(triangle_vertices.clone())
      .with_density(100.0)
      .with_friction(0.5)
      .with_restitution(0.15)
      .build(&mut physics_world, polygon_body)
      .expect("Failed to create polygon collider");
    collider_inits.push(ColliderRenderInit {
      body: polygon_body,
      shape: ColliderShape2D::ConvexPolygon {
        vertices: triangle_vertices,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.45, 0.95, 0.4, 1.0],
    });

    // Local rotation demo: a thin rectangle rotated in local space.
    let local_rotation_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.05, 0.55)
      .build(&mut physics_world)
      .expect("Failed to create local rotation body");
    Collider2DBuilder::rectangle(0.11, 0.03)
      .with_local_rotation(0.85)
      .with_density(100.0)
      .with_friction(0.4)
      .with_restitution(0.1)
      .build(&mut physics_world, local_rotation_body)
      .expect("Failed to create local rotation collider");
    collider_inits.push(ColliderRenderInit {
      body: local_rotation_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.11,
        half_height: 0.03,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.85,
      tint: [0.95, 0.6, 0.25, 1.0],
    });

    // Compound collider demo: a dumbbell made of two circles on one body.
    let compound_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(-0.02, 0.68)
      .build(&mut physics_world)
      .expect("Failed to create compound body");

    Collider2DBuilder::rectangle(0.09, 0.025)
      .with_density(100.0)
      .with_friction(0.25)
      .with_restitution(0.0)
      .build(&mut physics_world, compound_body)
      .expect("Failed to create compound center collider");
    collider_inits.push(ColliderRenderInit {
      body: compound_body,
      shape: ColliderShape2D::Rectangle {
        half_width: 0.09,
        half_height: 0.025,
      },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.55, 0.32, 0.72, 1.0],
    });

    for offset_x in [-0.07, 0.07] {
      Collider2DBuilder::circle(0.05)
        .with_offset(offset_x, 0.0)
        .with_density(100.0)
        .with_friction(0.25)
        .with_restitution(0.0)
        .build(&mut physics_world, compound_body)
        .expect("Failed to create compound circle collider");
      collider_inits.push(ColliderRenderInit {
        body: compound_body,
        shape: ColliderShape2D::Circle { radius: 0.05 },
        local_offset: [offset_x, 0.0],
        local_rotation: 0.0,
        tint: [0.75, 0.45, 0.95, 1.0],
      });
    }

    // Density demo: same size circles, different density (mass).
    let density_light_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.62, 0.65)
      .build(&mut physics_world)
      .expect("Failed to create density light body");
    Collider2DBuilder::circle(0.06)
      .with_density(100.0)
      .with_friction(0.0)
      .with_restitution(0.0)
      .build(&mut physics_world, density_light_body)
      .expect("Failed to create density light collider");
    collider_inits.push(ColliderRenderInit {
      body: density_light_body,
      shape: ColliderShape2D::Circle { radius: 0.06 },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.35, 0.75, 1.0, 1.0],
    });

    let density_heavy_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.82, 0.65)
      .build(&mut physics_world)
      .expect("Failed to create density heavy body");
    Collider2DBuilder::circle(0.06)
      // Keep the mass ratio small enough that a unit impulse produces visible
      // motion for both bodies.
      .with_density(250.0)
      .with_friction(0.0)
      .with_restitution(0.0)
      .build(&mut physics_world, density_heavy_body)
      .expect("Failed to create density heavy collider");
    collider_inits.push(ColliderRenderInit {
      body: density_heavy_body,
      shape: ColliderShape2D::Circle { radius: 0.06 },
      local_offset: [0.0, 0.0],
      local_rotation: 0.0,
      tint: [0.15, 0.35, 0.95, 1.0],
    });

    let mut shader_builder = ShaderBuilder::new();
    let vertex_shader = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "physics-colliders-2d".to_string(),
    });
    let fragment_shader = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "physics-colliders-2d".to_string(),
    });

    return Self {
      physics_world,
      physics_accumulator_seconds: 0.0,

      pending_impulse: false,
      impulse_cooldown_remaining_seconds: 0.0,
      density_light_body,
      density_heavy_body,

      vertex_shader,
      fragment_shader,
      mesh: None,
      render_pipeline_id: None,
      render_pass_id: None,
      colliders: Vec::new(),
      collider_inits,

      width: 1200,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Physics: 2D Colliders")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("Physics: 2D Colliders");
    })
    .with_component(move |runtime, demo: Colliders2DDemo| {
      return (runtime, demo);
    })
    .build();

  start_runtime(runtime);
}
