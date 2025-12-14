use iced::{Element, Length, Theme, Task};
use iced::widget::{column, container, text, button, row, scrollable, horizontal_space, rule};
use focusd_core::{db::Db, config::Config};

pub fn main() -> iced::Result {
    iced::application("Focusd Dashboard", FocusdApp::update, FocusdApp::view)
        .theme(|_| Theme::Dark)
        .run()
}

struct FocusdApp {
    db: Db,
    config: Config,
    total_time_str: String,
    // Store the raw list of apps: (Name, Seconds)
    apps_data: Vec<(String, i64)>,
}

#[derive(Debug, Clone)]
enum Message {
    LoadData,
}

impl FocusdApp {
    fn new() -> (Self, Task<Message>) {
        let db = Db::init().expect("DB Failed");
        let config = Config::load();
        
        let app = Self {
            db,
            config,
            total_time_str: "Loading...".to_string(),
            apps_data: Vec::new(),
        };
        
        (app, Task::perform(async {}, |_| Message::LoadData))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::LoadData => {
                match self.db.get_usage_since(0) { // 0 = Today
                    Ok(data) => {
                        // Calculate Total
                        let total_seconds: i64 = data.iter().map(|(_, s)| s).sum();
                        self.total_time_str = format_duration(total_seconds);
                        
                        // Store Data
                        self.apps_data = data;
                    }
                    Err(e) => self.total_time_str = format!("Error: {}", e),
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // 1. Create the App List view
        let mut apps_list = column![].spacing(10);

        if self.apps_data.is_empty() {
            apps_list = apps_list.push(text("No activity recorded today.").color([0.5, 0.5, 0.5]));
        } else {
            // Find max for progress bar scaling
            let max_val = self.apps_data.iter().map(|(_, s)| *s).max().unwrap_or(1) as f32;

            for (raw_name, seconds) in &self.apps_data {
                if raw_name.trim().is_empty() { continue; }

                // Apply Alias Lookup
                let display_name = self.config.alias.get(raw_name).unwrap_or(raw_name);
                
                // Duration String
                let dur = format_duration(*seconds);

                // Simple "Progress Bar" Logic
                let pct = (*seconds as f32 / max_val).clamp(0.0, 1.0);
                let bar_width = 200.0; // Max pixels width
                let actual_width = bar_width * pct;

                // Visual Row
                let row_item = row![
                    // Name
                    text(display_name).width(Length::Fill),
                    
                    // Visual Bar (A container with a background color)
                    container(text(" "))
                        .width(Length::Fixed(actual_width))
                        .height(Length::Fixed(10.0))
                        .style(|_| container::Style { 
                            background: Some(iced::Background::Color(iced::Color::from_rgb(0.2, 0.8, 0.8))),
                            border: iced::Border { radius: 5.0.into(), ..Default::default() },
                            ..Default::default()
                        }),
                        
                    horizontal_space().width(10),
                    
                    // Time
                    text(dur).size(14).color([0.7, 0.7, 0.7])
                ]
                .align_y(iced::Alignment::Center)
                .spacing(10);

                apps_list = apps_list.push(row_item);
            }
        }

        // 2. Main Layout
        container(
            column![
                // Header
                row![
                    text("Today's Focus").size(30),
                    horizontal_space().width(Length::Fill),
                    button("Refresh").on_press(Message::LoadData)
                ]
                .align_y(iced::Alignment::Center),

                // Big Total Number
                text(&self.total_time_str)
                    .size(60)
                    .color([0.4, 1.0, 0.4]), // Light green
                
                rule::Rule::horizontal(1), // Separator line

                // Scrollable Content
                scrollable(apps_list)
                    .height(Length::Fill)
            ]
            .spacing(20)
            .padding(20)
        )
        .center_x(Length::Fill)
        .into()
    }
}

// Helper to format "1h 30m"
fn format_duration(seconds: i64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    // Optional: show seconds if you want
    format!("{}h {}m", h, m)
}

impl Default for FocusdApp {
    fn default() -> Self {
        let (app, _) = Self::new();
        app
    }
}