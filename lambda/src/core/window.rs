use std::rc::Rc;

use winit::{
    dpi::{
        LogicalSize,
        PhysicalSize,
    },
    window::{
        Window as WinitWindow,
        WindowBuilder
    },
};

use crate::core::event_loop::{
    LambdaEventLoop,
};

/// The base window trait that every lambda window implementation must have to
/// work with lambda::core components.
pub trait Window{
    fn new() -> Self;
    fn redraw(&self);
    fn close(&self);
}

/// Metadata for Lambda window sizing that supports Copy and move operations.
#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub logical: LogicalSize<u32>,
    pub physical: PhysicalSize<u32>,
}

/// Construct WindowSize metdata from the window dimensions and scale factor of
/// the monitor being rendered to.
#[inline]
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

pub struct LambdaWindow {
    name: String,
    size: WindowSize,
    winit_window: Option<WinitWindow>,
}

impl LambdaWindow {
    /// Rebind a Lambda window to an event loop that will be attached to
    /// the new LambdaWindow returned by this object.
    pub fn with_event_loop(self, event_loop: &LambdaEventLoop) -> Self {
        let name = self.name.to_string();
        let winit_loop = event_loop.winit_loop_ref();
        let size = construct_window_size(
            [self.size.width, self.size.height],
            winit_loop.primary_monitor().unwrap_or(
                winit_loop.available_monitors().next().unwrap()).scale_factor());

        let winit_window = Some(WindowBuilder::new()
                .with_title(name.to_string())
                .with_inner_size(self.size.logical)
                .build(winit_loop)
                .expect("Failed to create a winit handle for LambdaWindow."));

        return LambdaWindow{
            name,
            size,
            winit_window,
        };
    }
}


impl Window for LambdaWindow {

    /// Returns a constructed LambdaWindow in it's default configuration state..
    fn new() -> Self {
        const DEFAULT_TITLE: &str = "lambda window";
        const DEFAULT_WINDOW_SIZE: [u32; 2]= [512, 512];

        let window_size = construct_window_size(
                DEFAULT_WINDOW_SIZE,
                1.0);

        return LambdaWindow{
            name: DEFAULT_TITLE.to_string(),
            size: window_size,
            winit_window: None,
        };
    }

    fn close(&self) { }

    fn redraw(&self) { 
        self.winit_window.as_ref().unwrap().request_redraw();
    }
}