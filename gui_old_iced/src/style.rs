use iced::{widget::{button, container, rule}, Color, Background, Border, Vector, Theme};

// === PALETTE (Hex -> RGB / 255.0) ===

// ~Graphite #353535
pub const C_BG: Color = Color { r: 53.0/255.0, g: 53.0/255.0, b: 53.0/255.0, a: 1.0 };

// ~Yale Blue #284b63
pub const C_MAYA: Color = Color { r: 40.0/255.0, g: 75.0/255.0, b: 99.0/255.0, a: 1.0 };

// ~Stormy Teal #3c6e71
pub const C_SKY: Color = Color { r: 60.0/255.0, g: 110.0/255.0, b: 113.0/255.0, a: 1.0 };

// ~Dust Grey #d9d9d9
pub const C_TEXT: Color = Color { r: 217.0/255.0, g: 217.0/255.0, b: 217.0/255.0, a: 1.0 };

// ~White #ffffff
pub const C_PETAL: Color = Color::WHITE;

// === TRANSLUCENT ===
pub const C_CARD_BG: Color = Color { r: 40.0/255.0, g: 75.0/255.0, b: 99.0/255.0, a: 0.25 }; 
pub const C_HOVER: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 0.05 };

// === STYLES ===

pub fn card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(C_CARD_BG)),
        border: Border {
            color: Color { a: 0.1, ..C_TEXT }, 
            width: 1.0,
            radius: 16.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::BLACK,
            offset: Vector::new(0.0, 8.0),
            blur_radius: 15.0,
        },
        ..Default::default()
    }
}

pub fn list_item(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.2))),
        border: Border { radius: 12.0.into(), ..Default::default() },
        ..Default::default()
    }
}

// === BUTTONS ===
pub fn tab_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(C_MAYA)),
        text_color: C_TEXT, 
        border: Border { radius: 20.0.into(), ..Default::default() },
        shadow: iced::Shadow { color: Color::BLACK, offset: Vector::new(0.0, 2.0), blur_radius: 5.0 },
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

// === FIX: This matches Fn(&Theme) -> rule::Style ===
pub fn separator(_theme: &Theme) -> rule::Style {
    rule::Style {
        color: Color::from_rgba(1.0, 1.0, 1.0, 0.1),
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}