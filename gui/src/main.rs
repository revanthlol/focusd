mod style;
mod chart;
mod app;

use iced::Theme;
use app::FocusdApp;

pub fn main() -> iced::Result {
    iced::application("Focusd Dashboard", FocusdApp::update, FocusdApp::view)
        .subscription(FocusdApp::subscription)
        .theme(|_| Theme::Dark)
        .run_with(FocusdApp::new) // <--- Fix: Initializes with data loading task!
}