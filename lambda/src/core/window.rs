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
    fn set_event_callback<Events>(&mut self, event_callback: impl Fn(Events) -> bool );
    fn get_size(&self) -> [u32; 2];
}

/// Metadata for Lambda window sizing.
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub logical: LogicalSize<u32>,
    pub physical: PhysicalSize<u32>,
}

pub struct LambdaWindow {
    name: String,
    size: WindowSize,
    event_loop: EventLoop<()>,
    winit_handle: winit::window::Window
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
        const default_title: &str = "lambda window";
        const default_window_size: [u32; 2]= [512, 512];
        let event_loop = winit::event_loop::EventLoop::new();

        let window_size = construct_window_size(
                default_window_size,
                event_loop.primary_monitor().unwrap().scale_factor());


        let winit_handle = winit::window::WindowBuilder::new()
                .with_title(default_title)
                .with_inner_size(window_size.logical)
                .build(&event_loop)
                .expect("Failed to create a winit handle for LambdaWindow.");

        event_loop.run(|event, _, control_flow| {
            use winit::event::{Event, WindowEvent};
            use winit::event_loop::ControlFlow;

            match event {
                Event::WindowEvent {event, ..} => match event {
                    WindowEvent::CloseRequested => {
                        *control_flow = ControlFlow::Exit
                    },
                    WindowEvent::Resized(dims) => {
                    }
                }
            }
        });

        // Compute the logical and physical window sizes using the screens
        // primary monitor.
        return LambdaWindow{
            name: default_title.to_string(),
            size: window_size,
            event_loop,
            winit_handle
        };
    }

    fn on_event(&mut self) {
    }

    fn on_update(&mut self) {}
    fn set_event_callback<LambdaEvents>(&mut self,
            event_callback: impl Fn(LambdaEvents) -> bool) {
        self.callback = event_callback;
    }

    fn get_size(&self) -> WindowSize {
        return self.size;
    }
}

impl LambdaWindow() {

}
