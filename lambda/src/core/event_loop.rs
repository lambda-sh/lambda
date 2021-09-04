use winit::event_loop::EventLoop;
use winit::monitor::MonitorHandle;
use winit::event::Event;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;

pub enum LambdaEvent{
    Initialized,
    Shutdown,
}

pub struct LambdaEventLoop {
    event_loop: Box<EventLoop<LambdaEvent>>
}

pub enum HardwareLookup {
    Monitor,
}

/// A wrapper over the Winit event loop that allows for reuse of their event
/// loops.
impl LambdaEventLoop {
    /// Creates a new Lambda event loop with the underlying event loop
    /// implementation allocated on the heap.
    pub fn new() -> Self {
        let event_loop = Box::new(EventLoop::<LambdaEvent>::with_user_event());
        return LambdaEventLoop{
            event_loop,
        };
    }

    pub fn run_in_main_thread(self) {
        self.event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(dims) => {},
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => { },
                    _ => (),
                },
                Event::MainEventsCleared => {},
                Event::RedrawRequested(_) => {
                }
                _ => (),
            }
        });
    }

    pub fn run_in_separate_thread(self) {
          self.event_loop.run(move |event, _, control_flow| {
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(dims) => {},
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => { },
                        _ => (),
                    },
                    Event::MainEventsCleared => {},
                    Event::RedrawRequested(_) => {
                    }
                    _ => (),
                }
            });
    }

    pub fn get_proxy(&self) {}

    pub fn from_winit(&self) -> &EventLoop<LambdaEvent> {
        return &self.event_loop;
    }


}
