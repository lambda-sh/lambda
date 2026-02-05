#![allow(clippy::needless_return)]

//! Example: Render to an offscreen target, then sample it to the surface.

use lambda::{
  component::Component,
  events::{
    EventMask,
    WindowEvent,
  },
  logging,
  render::{
    bind::{
      BindGroupBuilder,
      BindGroupLayout,
      BindGroupLayoutBuilder,
    },
    buffer::BufferBuilder,
    command::{
      RenderCommand,
      RenderDestination,
    },
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
    targets::offscreen::OffscreenTargetBuilder,
    texture::SamplerBuilder,
    vertex::{
      ColorFormat,
      Vertex,
      VertexAttribute,
      VertexBuilder,
      VertexElement,
    },
    viewport::ViewportBuilder,
    RenderContext,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

// ------------------------------ SHADER SOURCE --------------------------------

const POST_VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 2) in vec3 vertex_color; // uv packed into .xy

layout (location = 0) out vec2 v_uv;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
  v_uv = vertex_color.xy;
}
"#;

const POST_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec2 v_uv;
layout (location = 0) out vec4 fragment_color;

layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

void main() {
  fragment_color = texture(sampler2D(tex, samp), v_uv);
}
"#;

// --------------------------------- COMPONENT ---------------------------------

pub struct OffscreenPostExample {
  triangle_vs: Shader,
  triangle_fs: Shader,
  post_vs: Shader,
  post_fs: Shader,
  quad_mesh: Option<Mesh>,

  offscreen_pass: Option<ResourceId>,
  offscreen_pipeline: Option<ResourceId>,
  offscreen_target: Option<ResourceId>,

  post_pass: Option<ResourceId>,
  post_pipeline: Option<ResourceId>,
  post_bind_group: Option<ResourceId>,
  post_layout: Option<BindGroupLayout>,

  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for OffscreenPostExample {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching OffscreenPostExample");

    let surface_size = render_context.surface_size();
    let offscreen_target = OffscreenTargetBuilder::new()
      .with_color(
        render_context.surface_format(),
        surface_size.0,
        surface_size.1,
      )
      .with_label("offscreen-post-target")
      .build(render_context.gpu())
      .map_err(|e| format!("Failed to build offscreen target: {:?}", e))?;
    let offscreen_target_id =
      render_context.attach_offscreen_target(offscreen_target);

    let offscreen_pass =
      RenderPassBuilder::new().with_label("offscreen-pass").build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
      );

    let offscreen_pipeline = RenderPipelineBuilder::new()
      .with_label("offscreen-pipeline")
      .with_culling(CullingMode::None)
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &offscreen_pass,
        &self.triangle_vs,
        Some(&self.triangle_fs),
      );

    let post_pass = RenderPassBuilder::new().with_label("post-pass").build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    let post_layout = BindGroupLayoutBuilder::new()
      .with_sampled_texture(1)
      .with_sampler(2)
      .build(render_context.gpu());

    let sampler = SamplerBuilder::new()
      .linear_clamp()
      .with_label("offscreen-post-sampler")
      .build(render_context.gpu());

    let offscreen_ref =
      render_context.get_offscreen_target(offscreen_target_id);
    let post_bind_group = BindGroupBuilder::new()
      .with_layout(&post_layout)
      .with_texture(1, offscreen_ref.color_texture())
      .with_sampler(2, &sampler)
      .build(render_context.gpu());

    let quad_mesh = Self::build_fullscreen_quad_mesh();
    let quad_vertex_buffer =
      BufferBuilder::build_from_mesh(&quad_mesh, render_context.gpu())
        .map_err(|e| format!("Failed to build quad vertex buffer: {:?}", e))?;

    let post_pipeline = RenderPipelineBuilder::new()
      .with_label("post-pipeline")
      .with_culling(CullingMode::None)
      .with_layouts(&[&post_layout])
      .with_buffer(quad_vertex_buffer, quad_mesh.attributes().to_vec())
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &post_pass,
        &self.post_vs,
        Some(&self.post_fs),
      );

    self.offscreen_pass =
      Some(render_context.attach_render_pass(offscreen_pass));
    self.offscreen_pipeline =
      Some(render_context.attach_pipeline(offscreen_pipeline));
    self.offscreen_target = Some(offscreen_target_id);

    self.post_pass = Some(render_context.attach_render_pass(post_pass));
    self.post_pipeline = Some(render_context.attach_pipeline(post_pipeline));
    self.post_bind_group =
      Some(render_context.attach_bind_group(post_bind_group));
    self.post_layout = Some(post_layout);
    self.quad_mesh = Some(quad_mesh);

    let (width, height) = render_context.surface_size();
    self.width = width;
    self.height = height;

    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    if let WindowEvent::Resize { width, height } = event {
      self.width = *width;
      self.height = *height;
    }
    return Ok(());
  }

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Vec<RenderCommand> {
    self.ensure_offscreen_matches_surface(render_context);

    let offscreen_viewport =
      ViewportBuilder::new().build(self.width.max(1), self.height.max(1));
    let surface_viewport =
      ViewportBuilder::new().build(self.width.max(1), self.height.max(1));

    return vec![
      RenderCommand::BeginRenderPassTo {
        render_pass: self.offscreen_pass.expect("offscreen pass not set"),
        viewport: offscreen_viewport.clone(),
        destination: RenderDestination::Offscreen(
          self.offscreen_target.expect("offscreen target not set"),
        ),
      },
      RenderCommand::SetPipeline {
        pipeline: self.offscreen_pipeline.expect("offscreen pipeline not set"),
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![offscreen_viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![offscreen_viewport.clone()],
      },
      RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
      RenderCommand::BeginRenderPass {
        render_pass: self.post_pass.expect("post pass not set"),
        viewport: surface_viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: self.post_pipeline.expect("post pipeline not set"),
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group: self.post_bind_group.expect("post bind group not set"),
        dynamic_offsets: vec![],
      },
      RenderCommand::BindVertexBuffer {
        pipeline: self.post_pipeline.expect("post pipeline not set"),
        buffer: 0,
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![surface_viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![surface_viewport.clone()],
      },
      RenderCommand::Draw {
        vertices: 0..6,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl OffscreenPostExample {
  fn build_fullscreen_quad_mesh() -> Mesh {
    let vertices: [Vertex; 6] = [
      VertexBuilder::new()
        .with_position([-1.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([1.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([-1.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([-1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 1.0, 0.0])
        .build(),
    ];

    let mut mesh_builder = MeshBuilder::new();
    for v in vertices {
      mesh_builder.with_vertex(v);
    }

    return mesh_builder
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
  }

  fn ensure_offscreen_matches_surface(
    &mut self,
    render_context: &mut RenderContext,
  ) {
    let offscreen_id = match self.offscreen_target {
      Some(id) => id,
      None => return,
    };
    let post_layout = match self.post_layout.as_ref() {
      Some(layout) => layout,
      None => return,
    };
    let bind_group_id = match self.post_bind_group {
      Some(id) => id,
      None => return,
    };

    let surface_size = render_context.surface_size();
    let target_size = render_context.get_offscreen_target(offscreen_id).size();
    if target_size == surface_size {
      return;
    }

    let new_target = match OffscreenTargetBuilder::new()
      .with_color(
        render_context.surface_format(),
        surface_size.0,
        surface_size.1,
      )
      .with_label("offscreen-post-target")
      .build(render_context.gpu())
    {
      Ok(target) => target,
      Err(error) => {
        logging::error!("Failed to rebuild offscreen target: {:?}", error);
        return;
      }
    };

    if let Err(error) =
      render_context.replace_offscreen_target(offscreen_id, new_target)
    {
      logging::error!("Failed to replace offscreen target: {}", error);
      return;
    }

    let offscreen_ref = render_context.get_offscreen_target(offscreen_id);
    let sampler = SamplerBuilder::new()
      .linear_clamp()
      .with_label("offscreen-post-sampler")
      .build(render_context.gpu());
    let new_bind_group = BindGroupBuilder::new()
      .with_layout(post_layout)
      .with_texture(1, offscreen_ref.color_texture())
      .with_sampler(2, &sampler)
      .build(render_context.gpu());

    if let Err(error) =
      render_context.replace_bind_group(bind_group_id, new_bind_group)
    {
      logging::error!("Failed to replace post bind group: {}", error);
    }
  }
}

impl Default for OffscreenPostExample {
  fn default() -> Self {
    let triangle_vertex = VirtualShader::Source {
      source: include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/lambda-rs/assets/shaders/triangle.vert"
      ))
      .to_string(),
      kind: ShaderKind::Vertex,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    let triangle_fragment = VirtualShader::Source {
      source: include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../crates/lambda-rs/assets/shaders/triangle.frag"
      ))
      .to_string(),
      kind: ShaderKind::Fragment,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    let mut builder = ShaderBuilder::new();
    let triangle_vs = builder.build(triangle_vertex);
    let triangle_fs = builder.build(triangle_fragment);

    let post_vs = builder.build(VirtualShader::Source {
      source: POST_VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "offscreen-post".to_string(),
    });
    let post_fs = builder.build(VirtualShader::Source {
      source: POST_FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "offscreen-post".to_string(),
    });

    return OffscreenPostExample {
      triangle_vs,
      triangle_fs,
      post_vs,
      post_fs,
      quad_mesh: None,
      offscreen_pass: None,
      offscreen_pipeline: None,
      offscreen_target: None,
      post_pass: None,
      post_pipeline: None,
      post_bind_group: None,
      post_layout: None,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Offscreen Post Process")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("Offscreen Post Process");
    })
    .with_component(move |runtime, component: OffscreenPostExample| {
      return (runtime, component);
    })
    .build();

  start_runtime(runtime);
}
