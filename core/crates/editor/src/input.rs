#[derive(Clone, Debug)]
pub enum InputEvent {
    MouseDown { x: f64, y: f64 },
    MouseMove { x: f64, y: f64 },
    Key(char),
}
