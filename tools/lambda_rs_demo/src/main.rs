use lambda::{
  core::{
    component::{
      Component,
      RenderableComponent,
    },
    events::Event,
    kernel::start_kernel,
    render::{
      internal::RenderPassBuilder,
      shader::{
        ShaderBuilder,
        ShaderKind,
        VirtualShader,
      },
      RenderContextBuilder,
    },
  },
  kernels::LambdaKernelBuilder,
};

pub struct DemoComponent {}

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
impl RenderableComponent<lambda::core::events::Event> for DemoComponent {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
    println!("Attached the second layer to lambda");
  }

  fn on_render(
    self: &mut DemoComponent,
    render_context: &mut lambda::core::render::RenderContext,
    last_render: &std::time::Duration,
  ) {
    println!("Rendering the second layer");
  }

  fn on_detach(
    self: &mut DemoComponent,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
  }
}

impl DemoComponent {}

impl Default for DemoComponent {
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

    return DemoComponent {};
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

// These 40 lines of code create what you saw before
