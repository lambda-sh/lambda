use winit::{
    event_loop::EventLoop,
    dpi::LogicalSize,
    dpi::PhysicalSize,
};

pub enum LambdaEvents {
    AppTick,
    AppStartup,
    AppShutdown,
}


pub trait Window{
    fn new() -> Self;
    fn on_update(&mut self);
    fn on_event(&mut self);
}

/// Metadata for Lambda window sizing.
#[derive(Clone)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub logical: LogicalSize<u32>,
    pub physical: PhysicalSize<u32>,
}

pub struct LambdaWindow {
    name: String,
    size: WindowSize,
    event_loop: Box<EventLoop<()>>,
    winit_handle: Box<winit::window::Window>
}

/// Construct a WindowSize struct from the window dimensions and scale factor.
fn construct_window_size(
        window_size: [u32; 2], scale_factor: f64) -> WindowSize {
    let logical: LogicalSize<u32> = window_size.into();
    let physical: PhysicalSize<u32> = logical.to_physical(scale_factor);

    return WindowSize{
        width: window_size[0],
        height: window_size[1],
        logical,
        physical
    }
}

impl Window for LambdaWindow {

    /// Returns a constructed LambdaWindow in a default state.
    fn new() -> Self {
        const DEFAULT_TITLE: &str = "lambda window";
        const DEFAULT_WINDOW_SIZE: [u32; 2]= [512, 512];
        let event_loop = winit::event_loop::EventLoop::new();

        let window_size = construct_window_size(
                DEFAULT_WINDOW_SIZE,
                event_loop.primary_monitor().unwrap().scale_factor());


        let winit_handle = winit::window::WindowBuilder::new()
                .with_title(DEFAULT_TITLE)
                .with_inner_size(window_size.logical)
                .build(&event_loop)
                .expect("Failed to create a winit handle for LambdaWindow.");


        // Compute the logical and physical window sizes using the screens
        // primary monitor.
        return LambdaWindow{
            name: DEFAULT_TITLE.to_string(),
            size: window_size,
            event_loop: Box::new(event_loop),
            winit_handle: Box::new(winit_handle),
        };
    }

    fn on_event(&mut self) {
    }

    fn on_update(&mut self) {}
}
