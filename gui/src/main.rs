use iced::{
    widget::{column, container, text, button, row, scrollable, horizontal_space, rule, canvas},
    Element, Length, Theme, Task, Point, Size, Color, Rectangle, mouse, Background, Border, alignment
};
use focusd_core::{db::Db, config::Config};
use chrono::{Datelike, Local, NaiveDate, Duration};

pub fn main() -> iced::Result {
    iced::application("Focusd", FocusdApp::update, FocusdApp::view)
        .theme(|_| Theme::Dark)
        .run()
}

// ================= THEME CONSTANTS =================
// Black
const C_BG: Color = Color::from_rgb(0.0, 0.0, 0.0);
// Prussian Blue
const C_CARD: Color = Color::from_rgb(0.078, 0.129, 0.239);
// Orange
const C_ACCENT: Color = Color::from_rgb(0.988, 0.639, 0.066);
// Alabaster
const C_TEXT_SUB: Color = Color::from_rgb(0.898, 0.898, 0.898);
// White
const C_TEXT_MAIN: Color = Color::WHITE;
// Explicit White for charts
const C_WHITE: Color = Color::WHITE; 

// ================= STATE =================
struct FocusdApp {
    db: Db,
    config: Config,
    current_tab: Tab,
    reference_date: NaiveDate,
    total_display_str: String,
    apps_data: Vec<(String, i64)>,
    daily_chart_data: Vec<(String, i64)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab { Today, Week }

#[derive(Debug, Clone)]
enum Message {
    SwitchTab(Tab),
    Refresh,
    PreviousPeriod,
    NextPeriod,
    JumpToNow,
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
            reference_date: Local::now().date_naive(), 
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
                self.reference_date = Local::now().date_naive();
                Task::perform(async {}, |_| Message::Refresh)
            }
            Message::PreviousPeriod => {
                self.reference_date -= match self.current_tab {
                    Tab::Today => Duration::days(1),
                    Tab::Week => Duration::days(7),
                };
                Task::perform(async {}, |_| Message::Refresh)
            }
            Message::NextPeriod => {
                let now = Local::now().date_naive();
                let next_date = self.reference_date + match self.current_tab {
                    Tab::Today => Duration::days(1),
                    Tab::Week => Duration::days(7),
                };
                if next_date <= now {
                    self.reference_date = next_date;
                    Task::perform(async {}, |_| Message::Refresh)
                } else {
                    Task::none()
                }
            }
            Message::JumpToNow => {
                self.reference_date = Local::now().date_naive();
                Task::perform(async {}, |_| Message::Refresh)
            }
            Message::Refresh => {
                let (start, end) = match self.current_tab {
                    Tab::Today => (self.reference_date, self.reference_date),
                    Tab::Week => {
                        let weekday = self.reference_date.weekday();
                        let days_since_mon = weekday.num_days_from_monday();
                        let monday = self.reference_date - Duration::days(days_since_mon as i64);
                        let sunday = monday + Duration::days(6);
                        
                        let today = Local::now().date_naive();
                        let effective_end = if sunday > today { today } else { sunday };
                        
                        (monday, effective_end)
                    }
                };

                // 1. Fetch List
                match self.db.get_app_usage_range(start, end) {
                    Ok(data) => {
                        let total: i64 = data.iter().map(|(_, s)| s).sum();
                        self.total_display_str = format_duration(total);
                        self.apps_data = data;
                    }
                    Err(e) => self.total_display_str = format!("Error: {}", e),
                }

                // 2. Fetch Chart
                if self.current_tab == Tab::Week {
                    let mut chart_vec = Vec::new();
                    let weekday = self.reference_date.weekday();
                    let start_of_week = self.reference_date - Duration::days(weekday.num_days_from_monday() as i64);
                    let end_of_week = start_of_week + Duration::days(6);
                    let today = Local::now().date_naive();
                    
                    if let Ok(map) = self.db.get_daily_totals(start_of_week, today) {
                        let mut curr = start_of_week;
                        while curr <= end_of_week {
                            let val = *map.get(&curr.to_string()).unwrap_or(&0);
                            chart_vec.push((curr.format("%a").to_string(), val));
                            curr += Duration::days(1);
                        }
                    }
                    self.daily_chart_data = chart_vec;
                } else {
                    self.daily_chart_data.clear();
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        // --- BUTTONS ---
        let tab_btn = |label, tab| {
            let is_selected = self.current_tab == tab;
            button(text(label).size(14).center())
                .on_press(Message::SwitchTab(tab))
                .padding([6, 16])
                // FIX: Closure now accepts (_theme, _status) 2 args
                .style(move |_, _| {
                     if is_selected {
                        button::Style {
                            background: Some(Background::Color(C_ACCENT)), // Orange active
                            text_color: C_BG, // Black Text
                            border: Border { radius: 20.0.into(), ..Default::default() },
                            ..Default::default()
                        }
                     } else {
                         button::Style {
                            background: None,
                            text_color: C_TEXT_SUB,
                            ..Default::default()
                         }
                     }
                })
        };

        // --- NAVIGATION ---
        let date_label_str = match self.current_tab {
            Tab::Today => self.reference_date.format("%b %d, %Y").to_string(),
            Tab::Week => {
                let w = self.reference_date.weekday();
                let start_date = self.reference_date - Duration::days(w.num_days_from_monday() as i64);
                let end_date = start_date + Duration::days(6);
                format!("{} - {}", start_date.format("%b %d"), end_date.format("%b %d"))
            }
        };

        let nav_bar = row![
             button(text("Today").size(12).color(C_TEXT_SUB)).on_press(Message::JumpToNow).padding([4, 12]).style(style_button_ghost),
             horizontal_space().width(10),
             button(text("◄").size(16).color(C_ACCENT)).on_press(Message::PreviousPeriod).style(style_button_ghost),
             text(date_label_str).size(16).color(C_TEXT_MAIN).width(Length::Fixed(150.0)).center(),
             button(text("►").size(16).color(C_ACCENT)).on_press(Message::NextPeriod).style(style_button_ghost),
        ].align_y(alignment::Alignment::Center);

        // --- HEADER ---
        let header = row![
            column![
                 text("Focusd").size(26).font(iced::font::Font::MONOSPACE).color(C_TEXT_MAIN),
                 text("Productivity").size(12).color(C_ACCENT)
            ],
            horizontal_space().width(Length::Fill),
            column![
                 row![tab_btn("Day", Tab::Today), tab_btn("Week", Tab::Week)].spacing(5),
                 vertical_space(5),
                 nav_bar
            ].align_x(alignment::Horizontal::Right)
        ].align_y(alignment::Alignment::Center).padding([10, 20]);

        // --- TOTAL CARD ---
        let hero = container(column![
             text("Focused Time").size(14).color(C_TEXT_SUB),
             text(&self.total_display_str).size(56).color(C_TEXT_MAIN), 
        ].spacing(5).align_x(alignment::Horizontal::Center))
        .width(Length::Fill)
        .padding(30)
        .style(style_card); 

        // --- CHART SECTION ---
        let chart_section: Element<Message> = if self.current_tab == Tab::Week && !self.daily_chart_data.is_empty() {
             container(
                 canvas(InteractiveBarChart { data: self.daily_chart_data.clone() })
                    .width(Length::Fill).height(Length::Fixed(160.0))
             ).style(style_card).padding(20).into()
        } else {
             column![].into()
        };

        // --- APP LIST ---
        let mut list_col = column![].spacing(8);
        if self.apps_data.is_empty() {
             list_col = list_col.push(
                container(text("No data yet.").size(16).color(C_TEXT_SUB))
                .width(Length::Fill).center_x(Length::Fill).padding(30)
            );
        } else {
             let max_val = self.apps_data.iter().map(|(_, s)| *s).max().unwrap_or(1) as f32;
             
             for (raw_name, seconds) in &self.apps_data {
                  if raw_name.trim().is_empty() { continue; }
                  let name = self.config.alias.get(raw_name).unwrap_or(raw_name);
                  let dur = format_duration(*seconds); 
                  let pct = (*seconds as f32 / max_val).clamp(0.01, 1.0);
                  
                  let app_row = row![
                       // Name
                       text(name).width(Length::FillPortion(3)).color(C_TEXT_MAIN),
                       
                       // Visual Bar
                       column![
                           container(horizontal_space()).width(Length::FillPortion(2)), 
                           container(text(" ")).width(Length::Fixed(150.0 * pct)).height(6)
                             .style(|_| container::Style { 
                                  background: Some(Background::Color(C_ACCENT)), 
                                  border: Border { radius: 3.0.into(), ..Default::default() },
                                  ..Default::default()
                             })
                       ].width(Length::FillPortion(2)).align_x(alignment::Horizontal::Left),
                       
                       // Duration
                       text(dur).size(12).color(C_TEXT_SUB).width(Length::Shrink)
                  ].align_y(alignment::Alignment::Center).spacing(10);
                  
                  list_col = list_col.push(container(app_row).padding([12, 16]).style(style_card_transparent));
             }
        }

        // --- MAIN LAYOUT ---
        let content = column![
            header,
            // FIX: width is u16 (1), not float (1.0)
            rule::Rule::horizontal(1).style(|_| rule::Style { 
                color: C_CARD, 
                width: 1, 
                radius: 0.0.into(), 
                fill_mode: rule::FillMode::Full 
            }),
            
            scrollable(column![
                 hero,
                 chart_section,
                 list_col
            ].spacing(20).padding(20))
        ];

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_| container::Style { background: Some(Background::Color(C_BG)), ..Default::default() })
            .into()
    }
}

// ================= STYLES =================

fn vertical_space(height: u16) -> iced::widget::Space {
    iced::widget::Space::with_height(Length::Fixed(height as f32))
}

fn style_card(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(C_CARD)), // Prussian Blue
        border: Border { radius: 16.0.into(), ..Default::default() },
        text_color: Some(C_TEXT_MAIN),
        ..Default::default()
    }
}

fn style_card_transparent(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.09, 0.14, 0.25))), 
        border: Border { radius: 8.0.into(), ..Default::default() },
        ..Default::default()
    }
}

fn style_button_ghost(_theme: &Theme, status: button::Status) -> button::Style {
    match status {
        button::Status::Hovered => button::Style {
             text_color: C_WHITE,
             background: Some(Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.1))),
             border: Border { radius: 8.0.into(), ..Default::default() },
             ..Default::default()
        },
        _ => button::Style {
            background: None,
            text_color: C_TEXT_SUB,
            ..Default::default()
        }
    }
}

fn format_duration(seconds: i64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    
    if h > 0 {
        format!("{}h {}m {}s", h, m, s)
    } else if m > 0 {
        format!("{}m {}s", m, s)
    } else {
        format!("{}s", s)
    }
}

// ================= CHART LOGIC =================
struct InteractiveBarChart { data: Vec<(String, i64)> }

impl canvas::Program<Message> for InteractiveBarChart {
    type State = ();

    fn draw(&self, _state: &(), renderer: &iced::Renderer, _theme: &Theme, bounds: Rectangle, cursor: mouse::Cursor) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        if self.data.is_empty() { return vec![frame.into_geometry()]; }

        let max_val = self.data.iter().map(|(_, v)| *v).max().unwrap_or(3600).max(60) as f32;
        
        let padding_x = 10.0;
        let padding_bottom = 25.0;
        let gap = 15.0;
        
        let available_w = bounds.width - (padding_x * 2.0);
        let bar_count = self.data.len() as f32;
        let bar_width = (available_w - (gap * (bar_count - 1.0))) / bar_count;
        let max_draw_h = bounds.height - padding_bottom;
        
        let cursor_pos = cursor.position_in(bounds);

        for (i, (label, val)) in self.data.iter().enumerate() {
            let x = padding_x + (i as f32 * (bar_width + gap));
            let h = (*val as f32 / max_val) * max_draw_h;
            let y = bounds.height - padding_bottom - h;
            
            let rect = Rectangle { x, y, width: bar_width, height: h };
            let is_hovered = cursor_pos.map(|p| rect.contains(p)).unwrap_or(false);
            
            let color = if is_hovered { C_WHITE } else { C_ACCENT };
            frame.fill(&canvas::Path::rectangle(Point::new(x, y), Size::new(bar_width, h)), color);

            let mut txt = canvas::Text::from(label.clone());
            txt.color = C_TEXT_SUB;
            txt.size = 12.0.into();
            txt.position = Point::new(x + bar_width/2.0, bounds.height - 12.0);
            txt.horizontal_alignment = alignment::Horizontal::Center;
            frame.fill_text(txt);
            
            if is_hovered {
                let mut tool_txt = canvas::Text::from(format_duration(*val));
                tool_txt.color = C_WHITE;
                tool_txt.size = 14.0.into();
                tool_txt.position = Point::new(x + bar_width/2.0, y - 20.0);
                tool_txt.horizontal_alignment = alignment::Horizontal::Center;
                frame.fill_text(tool_txt);
            }
        }
        vec![frame.into_geometry()]
    }
}