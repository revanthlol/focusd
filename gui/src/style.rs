use iced::{widget::{button, container, rule}, Color, Background, Border, Vector, Theme};

// === PALETTE ===
pub const C_BG: Color = Color { r: 0.21, g: 0.215, b: 0.196, a: 1.0 };
pub const C_SKY: Color = Color { r: 0.325, g: 0.847, b: 0.984, a: 1.0 };
pub const C_MAYA: Color = Color { r: 0.4, g: 0.764, b: 1.0, a: 1.0 };
pub const C_TEXT: Color = Color { r: 0.86, g: 0.88, b: 0.91, a: 1.0 };
pub const C_PETAL: Color = Color { r: 0.83, g: 0.686, b: 0.725, a: 1.0 };

pub const C_CARD_BG: Color = Color { r: 0.25, g: 0.255, b: 0.24, a: 0.6 };
pub const C_HOVER: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.05 };

// === CARD STYLES ===
pub fn card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(C_CARD_BG)),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.05),
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 4.0),
            blur_radius: 10.0,
        },
        ..Default::default()
    }
}

pub fn list_item(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
        border: Border { radius: 12.0.into(), ..Default::default() },
        ..Default::default()
    }
}

// === BUTTONS ===
pub fn tab_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(C_MAYA)),
        text_color: C_BG,
        border: Border { radius: 20.0.into(), ..Default::default() },
        shadow: iced::Shadow { color: C_MAYA, offset: Vector::new(0.0, 0.0), blur_radius: 8.0 },
        ..Default::default()
    }
}

pub fn tab_inactive(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.05))),
        text_color: C_TEXT,
        border: Border { radius: 20.0.into(), ..Default::default() },
        ..Default::default()
    }
}

pub fn ghost_btn(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(C_HOVER)),
            text_color: C_SKY,
            border: Border { radius: 8.0.into(), ..Default::default() },
            ..Default::default()
        },
        _ => button::Style {
            background: None,
            text_color: C_TEXT,
            ..Default::default()
        }
    }
}

// === FIX: Added <'static> here ===
pub fn separator(_theme: &Theme) -> rule::Style {
    rule::Style {
        color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}