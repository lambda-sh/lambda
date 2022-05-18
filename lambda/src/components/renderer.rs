use core::mem::swap;
use std::collections::HashMap;

use lambda_platform::{
  gfx,
  gfx::gfx_hal_exports,
};

use crate::{
  components::Window,
  core::{
    component::Component,
    events::Event,
    render::shader::ShaderKind,
  },
};

/// Enums for How to load shaders into the
pub enum Shader {
  FromFile {
    name: Option<String>,
    path: String,
    kind: ShaderKind,
  },
  FromString {
    name: Option<String>,
    source: String,
    kind: ShaderKind,
  },
}

pub enum Resource {
  Shader(Shader),
}

pub enum OnAttachSteps {
  UploadShader(Shader),
}

pub enum OnDetachSteps {}
pub enum OnEventSteps {}
pub enum OnUpdateSteps {}

pub struct RenderPlan {
  on_attach: Vec<OnAttachSteps>,
  on_detach: Vec<OnDetachSteps>,
  on_event: Vec<OnEventSteps>,
  on_update: Vec<OnUpdateSteps>,
}

pub struct RendererBase<B: gfx_hal_exports::Backend> {
  instance: gfx::GfxInstance<B>,
  gpu: gfx::gpu::GfxGpu<B>,
  format: gfx_hal_exports::Format,
  extent: gfx_hal_exports::Extent2D,
  fences: HashMap<String, Fences<B>>,
  surfaces: HashMap<String, B::Surface>,

  graphic_pipelines: Vec<B::GraphicsPipeline>,
  pipeline_layouts: Vec<B::PipelineLayout>,
  render_passes: Vec<B::RenderPass>,
}

type PlatformAPI = gfx::api::RenderingAPI::Backend;

struct Fences<B: gfx_hal_exports::Backend> {
  submission_fence: B::Fence,
  rendering_semaphore: B::Semaphore,
}

type PlatformFences = Fences<PlatformAPI>;

/// The Renderer component utilizing the current platforms rendering backend
/// provided by
pub type Renderer = RendererBase<PlatformAPI>;

impl Renderer {
  pub fn new(name: &str, window: &Window) -> Self {
    let instance = gfx::GfxInstance::<PlatformAPI>::new(name);
    let mut surface = instance.create_surface(window.window_handle());
    let mut gpu = instance
      .open_primary_gpu(Some(&surface))
      .with_command_pool();

    // Create the image extent and initial frame buffer attachment description for rendering.
    let format = gpu.find_supported_color_format(&surface);
    let dimensions = window.dimensions();
    let (extent, _frame_buffer_attachment) = gpu
      .configure_swapchain_and_update_extent(
        &mut surface,
        format,
        [dimensions[0], dimensions[1]],
      );

    let mut surfaces = HashMap::new();
    surfaces.insert("Primary".to_string(), surface);
    let mut fences = HashMap::<String, PlatformFences>::new();

    let (submission_fence, rendering_semaphore) = gpu.create_access_fences();
    let fence_set = PlatformFences {
      submission_fence,
      rendering_semaphore,
    };

    fences.insert("Primary".to_string(), fence_set);

    return Self {
      instance,
      gpu,
      format,
      surfaces,
      extent,
      fences,
      graphic_pipelines: vec![],
      pipeline_layouts: vec![],
      render_passes: vec![],
    };
  }

  pub fn upload_render_plan(&mut self, plan: RenderPlan) {
    todo!();
  }
}

impl Component for Renderer {
  fn on_attach(&mut self) {
    println!("The rendering API has been attached to the current Runnable.")
  }

  /// When detaching the Renderer, it will deallocate all GPU resources that have been created.
  fn on_detach(&mut self) {
    println!("Destroying GPU resources allocated during run.");

    let mut empty_fences = HashMap::new();
    swap(&mut empty_fences, &mut self.fences);

    for (name, fence) in empty_fences.into_iter() {
      self.gpu.destroy_access_fences(
        fence.submission_fence,
        fence.rendering_semaphore,
      );
    }

    let mut pipeline_layouts = vec![];
    swap(&mut pipeline_layouts, &mut self.pipeline_layouts);

    for pipeline_layout in pipeline_layouts.into_iter() {
      self.gpu.destroy_pipeline_layout(pipeline_layout);
    }

    let mut render_passes = vec![];
    swap(&mut render_passes, &mut self.render_passes);

    for render_pass in render_passes.into_iter() {
      self.gpu.destroy_render_pass(render_pass);
    }

    let mut graphics_pipelines = vec![];
    swap(&mut graphics_pipelines, &mut self.graphic_pipelines);
    for pipeline in graphics_pipelines.into_iter() {
      self.gpu.destroy_graphics_pipeline(pipeline);
    }

    // Destroy command pool allocated on the GPU.
    self.gpu.destroy_command_pool();
    let mut surfaces = HashMap::new();
    swap(&mut surfaces, &mut self.surfaces);

    for (_surface_name, mut surface) in surfaces.into_iter() {
      // Unconfigure the swapchain and destroy the surface context.
      println!("Destroying Surface: {}", _surface_name);
      self.gpu.unconfigure_swapchain(&mut surface);
      self.instance.destroy_surface(surface);
    }

    println!("Destroyed all GPU resources");
  }

  fn on_event(&mut self, event: &Event) {}

  fn on_update(&mut self, last_frame: &std::time::Duration) {}
}
