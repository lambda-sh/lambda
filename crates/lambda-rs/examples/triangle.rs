#![allow(clippy::needless_return)]
use lambda::{
  component::Component,
  events::{
    ComponentEvent,
    Events,
    Key,
    WindowEvent,
  },
  render::{
    command::RenderCommand,
    pipeline,
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

pub struct DemoComponent {
  fragment_shader: Shader,
  vertex_shader: Shader,
  render_pass_id: Option<lambda::render::ResourceId>,
  render_pipeline_id: Option<lambda::render::ResourceId>,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for DemoComponent {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attached the demo component to the renderer");
    let render_pass = render_pass::RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    let pipeline = pipeline::RenderPipelineBuilder::new()
      .with_culling(pipeline::CullingMode::None)
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &render_pass,
        &self.vertex_shader,
        Some(&self.fragment_shader),
      );

    // Attach the render pass and pipeline to the render context
    self.render_pass_id = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline_id = Some(render_context.attach_pipeline(pipeline));

    logging::info!("Attached the DemoComponent.");
    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    self: &mut DemoComponent,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_event(
    self: &mut DemoComponent,
    event: Events,
  ) -> Result<ComponentResult, String> {
    match event {
      Events::Runtime { event, issued_at } => match event {
        lambda::events::RuntimeEvent::Shutdown => {
          logging::info!("Shutting down the runtime");
        }
        _ => {}
      },
      Events::Window { event, issued_at } => match event {
        WindowEvent::Resize { width, height } => {
          logging::info!("Window resized to {}x{}", width, height);
          self.width = width;
          self.height = height;
        }
        WindowEvent::Close => {
          logging::info!("Window closed");
        }
      },
      Events::Keyboard { event, issued_at } => match event {
        Key::Pressed {
          scan_code,
          virtual_key,
        } => {
          logging::debug!("Key pressed: {:?}", virtual_key);
        }
        Key::Released {
          scan_code,
          virtual_key,
        } => {
          logging::debug!("Key released: {:?}", virtual_key);
        }
        Key::ModifierPressed {
          modifier,
          virtual_key,
        } => {
          logging::debug!("Modifier pressed: {:?}", virtual_key);
        }
      },
      Events::Component { event, issued_at } => match event {
        ComponentEvent::Attached { name } => {
          logging::debug!("Component attached: {:?}", name);
        }
        ComponentEvent::Detached { name } => {
          logging::debug!("Component detached: {:?}", name);
        }
      },
      _ => {}
    };
    return Ok(ComponentResult::Success);
  }

  fn on_update(
    self: &mut DemoComponent,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    match last_frame.as_millis() > 20 {
      true => {
        logging::warn!("Last frame took {}ms", last_frame.as_millis());
      }
      false => {}
    };
    return Ok(ComponentResult::Success);
  }
  fn on_render(
    self: &mut DemoComponent,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    // Begin the pass first, then set pipeline/state inside
    return vec![
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass_id
          .expect("No render pass attached to the component"),
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: self
          .render_pipeline_id
          .expect("No pipeline attached to the component"),
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl DemoComponent {}

impl Default for DemoComponent {
  /// Load in shaders upon creation.

  fn default() -> Self {
    // Specify virtual shaders to use for rendering
    let triangle_vertex = VirtualShader::Source {
      source: include_str!("../assets/shaders/triangle.vert").to_string(),
      kind: ShaderKind::Vertex,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    let triangle_fragment = VirtualShader::Source {
      source: include_str!("../assets/shaders/triangle.frag").to_string(),
      kind: ShaderKind::Fragment,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    // Create a shader builder to compile the shaders.
    let mut builder = ShaderBuilder::new();
    let vs = builder.build(triangle_vertex);
    let fs = builder.build(triangle_fragment);

    return DemoComponent {
      vertex_shader: vs,
      fragment_shader: fs,
      render_pass_id: None,
      render_pipeline_id: None,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("2D Triangle Demo")
    .with_renderer_configured_as(move |render_context_builder| {
      return render_context_builder.with_render_timeout(1_000_000_000);
    })
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("2D Triangle Window");
    })
    .with_component(move |runtime, demo: DemoComponent| {
      return (runtime, demo);
    })
    .build();

  start_runtime(runtime);
}
