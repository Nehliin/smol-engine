#![allow(dead_code)]

pub mod directional_light;
pub mod point_light;
pub mod spotlight;

pub enum Strength {
    Weak,

    Medium,

    Strong,
}

impl Strength {
    pub fn get_values(&self) -> (f32, f32, f32) {
        match self {
            Strength::Weak => (1.0, 0.22, 0.2),
            Strength::Medium => (1.0, 0.09, 0.032),
            Strength::Strong => (1.0, 0.045, 0.0075),
        }
    }
}
