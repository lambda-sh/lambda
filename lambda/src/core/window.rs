
pub trait Window  {
    fn new() -> Self;
    fn init(&mut self);
    fn on_update(&mut self);
    fn register_event_listener(&mut self);
    fn get_size(&self) -> [u32; 2];
}

pub struct LambdaWindow {
    name: String,
    size: [u32; 2]
}

impl Window for LambdaWindow {
    fn new() -> Self {
        return LambdaWindow{
            name: String::from("lambda window"),
            size: [512, 512]
        };
    }
    fn init(&mut self) {}
    fn on_update(&mut self) {}
    fn register_event_listener(&mut self) {}

    fn get_size(&self) -> [u32; 2] {
        return self.size;
    }
}
