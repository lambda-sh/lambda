use std::rc::Rc;

use lambda::{
  core::{
    component::{
      Component,
      RenderableComponent,
    },
    events::Event,
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
      RenderContextBuilder,
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

impl Component<Event> for DemoComponent {
  fn on_attach(&mut self) {
    println!("Attached the first layer to lambda");
  }

  fn on_detach(self: &mut DemoComponent) {}

  fn on_event(self: &mut DemoComponent, event: &lambda::core::events::Event) {}

  fn on_update(self: &mut DemoComponent, last_frame: &std::time::Duration) {
    println!(
      "This layer was last updated: {} nanoseconds ago",
      last_frame.as_nanos()
    );

    println!(
      "This layer was last updated: {} milliseconds ago",
      last_frame.as_millis()
    );
  }
}

/// Implement rendering for the component.
impl RenderableComponent<Event> for DemoComponent {
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
    render_context: &mut lambda::core::render::RenderContext,
    last_render: &std::time::Duration,
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
            "No render pipeline set while trying to issue a render command",
          )
          .clone(),
      },
      RenderCommand::BeginRenderPass {
        render_pass: self.render_pass.as_ref().unwrap().clone(),
        viewport: viewport.clone(),
      },
      RenderCommand::Draw { vertices: 0..3 },
    ];
  }

  fn on_renderer_detached(
    self: &mut DemoComponent,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
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
      name: "triangle".to_string(),
      entry_point: "main".to_string(),
    };

    let triangle_fragment = VirtualShader::Source {
      source: include_str!("../assets/triangle.frag").to_string(),
      kind: ShaderKind::Fragment,
      name: "triangle".to_string(),
      entry_point: "main".to_string(),
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

/// This function demonstrates how to configure the renderer that comes with
/// the LambdaKernel. This is where you can upload shaders, configure render
/// passes, and generally allocate the resources you need from a completely safe
/// Rust API.
fn configure_renderer(builder: RenderContextBuilder) -> RenderContextBuilder {
  return builder;
}

fn main() {
  let kernel = LambdaKernelBuilder::new("Lambda 2D Demo")
    .configure_renderer(configure_renderer)
    .build()
    .with_component(move |kernel, demo: DemoComponent| {
      return (kernel, demo);
    });

  start_kernel(kernel);
}
