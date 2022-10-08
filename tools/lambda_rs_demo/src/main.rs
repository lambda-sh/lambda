use std::rc::Rc;

use lambda::{
  core::{
    component::{
      Component,
      RenderableComponent,
    },
    events::Events,
    kernel::start_kernel,
    render::{
      command::RenderCommand,
      pipeline::{
        self,
        RenderPipeline,
      },
      render_pass::{
        self,
        RenderPass,
      },
      shader::{
        Shader,
        ShaderBuilder,
        ShaderKind,
        VirtualShader,
      },
      viewport,
    },
  },
  kernels::LambdaKernelBuilder,
};

pub struct DemoComponent {
  triangle_vertex: Shader,
  vertex_shader: Shader,
  render_pass: Option<Rc<RenderPass>>,
  render_pipeline: Option<Rc<RenderPipeline>>,
}

impl Component<Events> for DemoComponent {
  fn on_attach(&mut self) {
    println!("Attached the first layer to lambda");
  }

  fn on_detach(self: &mut DemoComponent) {}

  fn on_event(self: &mut DemoComponent, _event: &lambda::core::events::Events) {
  }

  fn on_update(self: &mut DemoComponent, last_frame: &std::time::Duration) {
    println!(
      "This component was last updated: {} nanoseconds/{} milliseconds ago",
      last_frame.as_nanos(),
      last_frame.as_millis()
    );
  }
}

/// Implement rendering for the component.
impl RenderableComponent<Events> for DemoComponent {
  fn on_renderer_attached(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
    println!("Attached the demo component to the renderer");
    let render_pass =
      Rc::new(render_pass::RenderPassBuilder::new().build(&render_context));

    self.render_pass = Some(render_pass.clone());

    let pipeline = Rc::new(pipeline::RenderPipelineBuilder::new().build(
      render_context,
      &self.render_pass.as_ref().unwrap(),
      &self.vertex_shader,
      &self.triangle_vertex,
    ));

    self.render_pipeline = Some(pipeline.clone());
  }

  fn on_render(
    self: &mut DemoComponent,
    _render_context: &mut lambda::core::render::RenderContext,
    _last_render: &std::time::Duration,
  ) -> Vec<RenderCommand> {
    let viewport = viewport::ViewportBuilder::new().build(800, 600);

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
          .render_pipeline
          .as_ref()
          .expect(
            "No render pipeline set while trying to issue a render command.",
          )
          .clone(),
      },
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass
          .as_ref()
          .expect("Cannot begin the render pass when it doesn't exist.")
          .clone(),
        viewport: viewport.clone(),
      },
      RenderCommand::Draw { vertices: 0..3 },
    ];
  }

  fn on_renderer_detached(
    self: &mut DemoComponent,
    _render_context: &mut lambda::core::render::RenderContext,
  ) {
    println!("Detached the demo component from the renderer");
  }
}

impl DemoComponent {}

impl Default for DemoComponent {
  /// Load in shaders upon creation.

  fn default() -> Self {
    // Specify virtual shaders to use for rendering
    let triangle_vertex = VirtualShader::Source {
      source: include_str!("../assets/triangle.vert").to_string(),
      kind: ShaderKind::Vertex,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    let triangle_fragment = VirtualShader::Source {
      source: include_str!("../assets/triangle.frag").to_string(),
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
      triangle_vertex: fs,
      render_pass: None,
      render_pipeline: None,
    };
  }
}

fn main() {
  let kernel = LambdaKernelBuilder::new("Lambda 2D Demo")
    .with_renderer(move |render_context_builder| {
      return render_context_builder;
    })
    .with_component(move |kernel, demo: DemoComponent| {
      return (kernel, demo);
    })
    .build();

  start_kernel(kernel);
}
