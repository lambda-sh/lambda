use lambda::{
  core::{
    component::Component,
    events::{
      ComponentEvent,
      Events,
      KeyEvent,
      WindowEvent,
    },
    runtime::start_runtime,
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
  runtimes::ApplicationRuntimeBuilder,
};

pub struct DemoComponent {
  fragment_shader: Shader,
  vertex_shader: Shader,
  render_pass_id: Option<lambda::render::ResourceId>,
  render_pipeline_id: Option<lambda::render::ResourceId>,
  width: u32,
  height: u32,
}

impl Component for DemoComponent {
  fn on_attach(&mut self, render_context: &mut RenderContext) {
    println!("Attached the demo component to the renderer");
    let render_pass =
      render_pass::RenderPassBuilder::new().build(&render_context);

    let pipeline = pipeline::RenderPipelineBuilder::new().build(
      render_context,
      &render_pass,
      &self.vertex_shader,
      Some(&self.fragment_shader),
    );

    // Attach the render pass and pipeline to the render context
    self.render_pass_id = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline_id = Some(render_context.attach_pipeline(pipeline));

    println!("Attached the DemoComponent.");
  }

  fn on_detach(self: &mut DemoComponent, render_context: &mut RenderContext) {}

  fn on_event(self: &mut DemoComponent, event: Events) {
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
      Events::Keyboard { event, issued_at } => match event {
        KeyEvent::KeyPressed {
          scan_code,
          virtual_key,
        } => {
          println!("Key pressed: {:?}", virtual_key);
        }
        KeyEvent::KeyReleased {
          scan_code,
          virtual_key,
        } => {
          println!("Key released: {:?}", virtual_key);
        }
        KeyEvent::ModifierPressed {
          modifier,
          virtual_key,
        } => {
          println!("Modifier pressed: {:?}", virtual_key);
        }
      },
      Events::Component { event, issued_at } => match event {
        ComponentEvent::Attached { name } => {
          println!("Component attached: {:?}", name);
        }
        ComponentEvent::Detached { name } => {
          println!("Component detached: {:?}", name);
        }
      },
      _ => {}
    }
  }

  fn on_update(self: &mut DemoComponent, last_frame: &std::time::Duration) {
    match last_frame.as_millis() > 20 {
      true => {
        println!("[WARN] Last frame took {}ms", last_frame.as_millis());
      }
      false => {}
    }
  }
  fn on_render(
    self: &mut DemoComponent,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    // This array of commands will be executed in linear order
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
        pipeline: self
          .render_pipeline_id
          .expect("No pipeline attached to the component"),
      },
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass_id
          .expect("No render pass attached to the component"),
        viewport: viewport.clone(),
      },
      RenderCommand::Draw { vertices: 0..3 },
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
