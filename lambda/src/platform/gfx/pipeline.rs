use gpu;

/// A builder for
struct GfxPipelineBuilder<'a, B: gfx_hal::Backend> {
  gpu: &'a gpu::GfxGpu<B>,
  render_pass: &'a B::RenderPass,
}

impl<'a, B: gfx_hal::Backend> GfxPipeline<'a, B> {
  /// Return a new Pipeline Builder
  fn new(
    gpu: &'a gpu::GfxGpu<B>,
    render_pass: &'a B::RenderPass,
    pipeline_layout: Option<&'a B::PipelineLayout>,
  ) -> Self {
    let layout = match pipeline_layout {
      Some(layout) => layout,
      None => unsafe {
        return gpu.create_pipeline_layout();
      },
    };

    return Self { gpu, render_pass };
  }
}

unsafe fn make_pipeline<B: gfx_hal::Backend>() {}
