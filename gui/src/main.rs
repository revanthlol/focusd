use iced::{
    widget::{column, container, text, button, row, scrollable, horizontal_space, rule, canvas},
    Element, Length, Theme, Task, Point, Size, Color, Rectangle, mouse
};
use focusd_core::{db::Db, config::Config};
use chrono::{Datelike, Local, Duration};

pub fn main() -> iced::Result {
    iced::application("Focusd Dashboard", FocusdApp::update, FocusdApp::view)
        .theme(|_| Theme::Dark)
        .run()
}

// ================= STATE =================
struct FocusdApp {
    db: Db,
    config: Config,
    current_tab: Tab,
    total_display_str: String,
    apps_data: Vec<(String, i64)>,
    daily_chart_data: Vec<(String, i64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Today,
    Week,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchTab(Tab),
    Refresh,
}

impl Default for FocusdApp {
    fn default() -> Self {
        let (app, _) = FocusdApp::new();
        app
    }
}

impl FocusdApp {
    fn new() -> (Self, Task<Message>) {
        let db = Db::init().expect("DB Failed");
        let config = Config::load();

        let app = Self {
            db,
            config,
            current_tab: Tab::Today,
            total_display_str: "Loading...".to_string(),
            apps_data: Vec::new(),
            daily_chart_data: Vec::new(),
        };

        (app, Task::perform(async {}, |_| Message::Refresh))
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.current_tab = tab;
                Task::perform(async {}, |_| Message::Refresh)
            }

            Message::Refresh => {
                let today = Local::now().date_naive();

                let (start, end) = match self.current_tab {
                    Tab::Today => (today, today),
                    Tab::Week => {
                        let days = today.weekday().num_days_from_monday();
                        let monday = today - Duration::days(days as i64);
                        (monday, today)
                    }
                };

                // App list
                match self.db.get_app_usage_range(start, end) {
                    Ok(data) => {
                        let total: i64 = data.iter().map(|(_, s)| s).sum();
                        self.total_display_str = format_duration(total);
                        self.apps_data = data;
                    }
                    Err(e) => self.total_display_str = format!("Err: {}", e),
                }

                // Chart (week only)
                if self.current_tab == Tab::Week {
                    let mut out = Vec::new();
                    let map = self.db.get_daily_totals(start, end).unwrap_or_default();

                    let mut curr = start;
                    while curr <= end {
                        let val = *map.get(&curr.to_string()).unwrap_or(&0);
                        out.push((curr.format("%a").to_string(), val));
                        curr += Duration::days(1);
                    }

                    self.daily_chart_data = out;
                } else {
                    self.daily_chart_data.clear();
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let tab_btn = |label, tab| {
            let mut btn = button(text(label).size(14))
                .on_press(Message::SwitchTab(tab));

            if self.current_tab != tab {
                btn = btn.style(button::secondary);
            }

            btn
        };

        let header = row![
            text("Focusd").size(24).font(iced::font::Font::MONOSPACE),
            horizontal_space().width(Length::Fill),
            tab_btn("Today", Tab::Today),
            horizontal_space().width(10),
            tab_btn("This Week", Tab::Week),
            horizontal_space().width(10),
            button("Refresh").on_press(Message::Refresh).style(button::text),
        ]
        .align_y(iced::Alignment::Center)
        .padding(10);

        let chart_section: Element<Message> =
            if self.current_tab == Tab::Week && !self.daily_chart_data.is_empty() {
                container(
                    canvas(SimpleBarChart {
                        data: self.daily_chart_data.clone(),
                    })
                    .width(Length::Fill)
                    .height(Length::Fixed(150.0)),
                )
                .padding(20)
                .into()
            } else {
                column![].into()
            };

        let mut apps_col = column![].spacing(8);

        if self.apps_data.is_empty() {
            apps_col = apps_col.push(
                text("No data.")
                    .size(16)
                    .color([0.5, 0.5, 0.5]),
            );
        } else {
            let max_val =
                self.apps_data.iter().map(|(_, s)| *s).max().unwrap_or(1) as f32;

            for (raw, seconds) in &self.apps_data {
                if raw.trim().is_empty() {
                    continue;
                }

                let name = self.config.alias.get(raw).unwrap_or(raw);
                let dur = format_duration(*seconds);
                let pct = (*seconds as f32 / max_val).clamp(0.01, 1.0);

                apps_col = apps_col.push(
                    row![
                        text(name.clone()).width(Length::FillPortion(3)),
                        container(horizontal_space()).width(Length::FillPortion(1)),
                        container(text(" "))
                            .width(Length::Fixed(180.0 * pct))
                            .height(8)
                            .style(|_| container::Style {
                                background: Some(iced::Background::Color(
                                    Color::from_rgb(0.0, 0.7, 0.9),
                                )),
                                border: iced::Border {
                                    radius: 4.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        horizontal_space().width(10),
                        text(dur)
                            .size(12)
                            .color([0.7, 0.7, 0.7])
                            .width(Length::Shrink),
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(5),
                );
            }
        }

        let content = column![
            header,
            rule::Rule::horizontal(1),
            container(
                text(&self.total_display_str)
                    .size(40)
                    .color([0.5, 1.0, 0.5]),
            )
            .center_x(Length::Fill),
            chart_section,
            scrollable(container(apps_col).padding(20)),
        ]
        .spacing(10);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// ================= HELPERS =================
fn format_duration(seconds: i64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    if h > 0 {
        format!("{}h {}m", h, m)
    } else {
        format!("{}m", m)
    }
}

// ================= CHART =================
struct SimpleBarChart {
    data: Vec<(String, i64)>,
}

impl canvas::Program<Message> for SimpleBarChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {

        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let count = self.data.len();
        if count == 0 {
            return vec![frame.into_geometry()];
        }

        let max_val = self.data.iter()
            .map(|(_, v)| *v)
            .max()
            .unwrap_or(3600)
            .max(60) as f32;

        let gap = 10.0;
        let bar_width =
            (bounds.width - gap * (count as f32 - 1.0)) / count as f32;

        for (i, (label, val)) in self.data.iter().enumerate() {
            let x = i as f32 * (bar_width + gap);
            let h = (*val as f32 / max_val) * (bounds.height - 20.0);
            let y = bounds.height - h - 20.0;

            let bar = canvas::Path::rectangle(
                Point::new(x, y),
                Size::new(bar_width, h),
            );
            frame.fill(&bar, Color::from_rgb(0.0, 0.6, 1.0));

            let mut t = canvas::Text::from(label.clone());
            t.color = Color::WHITE;
            t.position = Point::new(x + bar_width / 4.0, bounds.height - 15.0);
            t.size = 12.0.into();
            frame.fill_text(t);
        }

        vec![frame.into_geometry()]
    }
}
