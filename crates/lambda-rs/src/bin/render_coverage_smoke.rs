use lambda::render::{
  command::RenderCommand,
  pipeline::RenderPipelineBuilder,
  render_pass::RenderPassBuilder,
  shader::{
    ShaderBuilder,
    ShaderKind,
    VirtualShader,
  },
  viewport::ViewportBuilder,
  RenderContextBuilder,
};

fn main() {
  // This binary is intended to be executed under `cargo llvm-cov run` to
  // collect coverage for window/surface-backed rendering paths that cannot be
  // exercised from unit tests on macOS (winit requires main-thread event loop
  // creation).
  //
  // If the environment cannot create a window/surface (for example, a
  // headless CI runner), exit successfully rather than failing the entire
  // coverage job.

  let attempt = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
    let mut event_loop = lambda_platform::winit::LoopBuilder::new()
      .build::<lambda::events::Events>();
    let window = lambda::render::window::WindowBuilder::new()
      .with_name("lambda-render-coverage-smoke")
      .with_dimensions(64, 64)
      .with_vsync(true)
      .build(&mut event_loop);

    // Touch window wrapper accessors to increase coverage in `render/window.rs`.
    let _ = window.window_handle();
    let _ = window.dimensions();
    let _ = window.vsync_requested();
    window.redraw();

    let mut render_context =
      RenderContextBuilder::new("lambda-render-coverage")
        .with_vsync(true)
        .build(&window)
        .expect("build render context");

    let pass = RenderPassBuilder::new().with_label("coverage-pass").build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );
    let pass_id = render_context.attach_render_pass(pass);

    let vert_path = format!(
      "{}/assets/shaders/triangle.vert",
      env!("CARGO_MANIFEST_DIR")
    );
    let frag_path = format!(
      "{}/assets/shaders/triangle.frag",
      env!("CARGO_MANIFEST_DIR")
    );
    let mut shaders = ShaderBuilder::new();
    let vs = shaders.build(VirtualShader::File {
      path: vert_path,
      kind: ShaderKind::Vertex,
      name: "triangle-vert".to_string(),
      entry_point: "main".to_string(),
    });
    let fs = shaders.build(VirtualShader::File {
      path: frag_path,
      kind: ShaderKind::Fragment,
      name: "triangle-frag".to_string(),
      entry_point: "main".to_string(),
    });

    let pipeline = RenderPipelineBuilder::new()
      .with_label("coverage-pipeline")
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        render_context.get_render_pass(pass_id),
        &vs,
        Some(&fs),
      );
    let pipeline_id = render_context.attach_pipeline(pipeline);

    let viewport = ViewportBuilder::new().build(64, 64);

    render_context.render(vec![
      RenderCommand::BeginRenderPass {
        render_pass: pass_id,
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: pipeline_id,
      },
      RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ]);

    // Exercise resize + reconfigure.
    render_context.resize(32, 32);
    let viewport2 = ViewportBuilder::new().build(32, 32);
    render_context.render(vec![
      RenderCommand::BeginRenderPass {
        render_pass: pass_id,
        viewport: viewport2,
      },
      RenderCommand::SetPipeline {
        pipeline: pipeline_id,
      },
      RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ]);
  }));

  if attempt.is_err() {
    return;
  }
}
