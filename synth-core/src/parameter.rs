use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: &'static str,
    pub value: f32,
    pub delta: f32,
    pub range: (f32, f32),
    render: fn(f32) -> String,
}

impl Parameter {
    pub fn new(
        name: &'static str,
        initial: f32,
        delta: f32,
        range: (f32, f32),
        render: fn(f32) -> String,
    ) -> Self {
        Parameter {
            name,
            value: initial,
            delta,
            range,
            render,
        }
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn render_value(&self) -> String {
        (self.render)(self.value)
    }

    pub fn set_value(&mut self, new_value: f32) {
        self.value = new_value;
    }
}

pub enum ParameterChange {
    Increment,
    Decrement,
}

pub trait UserParameters {
    fn get_parameters(&self) -> Vec<Parameter>;
    fn update_parameter(&mut self, index: usize, change: ParameterChange) -> Option<Parameter>;
}
