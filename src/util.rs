/// Color uses f32 because the extra precision from f64 is pointless.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color3 {
    red: f32,
    green: f32,
    blue: f32,
}

impl Color3 {
    pub fn new(red: f32, green: f32, blue: f32) -> Self {
        Self {
            red: Self::check_component("red", red),
            green: Self::check_component("green", green),
            blue: Self::check_component("blue", blue),
        }
    }

    /// Ensure that the given color component is between 0 and 1 (inclusive).
    /// If it is, return the given value. If not, panic.
    fn check_component(component_name: &str, value: f32) -> f32 {
        if value < 0.0 || value > 1.0 {
            panic!(
                "Color component {} must be in [0.0, 1.0], but was {}",
                component_name, value
            );
        }
        value
    }

    pub fn red(&self) -> f32 {
        self.red
    }

    pub fn green(&self) -> f32 {
        self.green
    }

    pub fn blue(&self) -> f32 {
        self.blue
    }
}
