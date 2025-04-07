use bevy::color::Color;

#[derive(Copy, Clone, PartialEq)]
pub enum Size {
    Small,
    Medium,
    Large,
}

impl Size {
    pub const fn to_octave(&self) -> i32 {
        match self {
            Size::Small => 4,
            Size::Medium => 3,
            Size::Large => 2,
        }
    }

    pub const fn to_radius(&self) -> f32 {
        match self {
            Size::Small => 10.0,
            Size::Medium => 15.0,
            Size::Large => 25.0,
        }
    }

    pub const fn to_color(&self) -> Color {
        match self {
            Size::Small => Color::linear_rgb(1.8, 0.3, 0.3),
            Size::Medium => Color::linear_rgb(1.5, 1.3, 0.3),
            Size::Large => Color::linear_rgb(0.2, 0.2, 2.3),
        }
    }

    pub fn increment(&self) -> Size {
        match self {
            Size::Small => Size::Medium,
            Size::Medium => Size::Large,
            Size::Large => Size::Large,
        }
    }

    pub fn decrement(&self) -> Size {
        match self {
            Size::Small => Size::Small,
            Size::Medium => Size::Small,
            Size::Large => Size::Medium,
        }
    }
}
