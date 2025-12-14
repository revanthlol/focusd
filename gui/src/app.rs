use iced::{
    // FIXED: Added 'rule' and 'canvas' back to this list
    widget::{column, container, text, button, row, scrollable, horizontal_space, rule, canvas},
    Element, Length, Task, Background, Border, Subscription
};
use std::time::Duration as StdDur;
use focusd_core::{db::Db, config::Config};
use chrono::{Datelike, Local, NaiveDate, Duration};

use crate::style::*; 
use crate::chart::AnimatedChart;

pub struct FocusdApp {
    db: Db,
    config: Config,
    current_tab: Tab,
    ref_date: NaiveDate,
    total_str: String,
    apps: Vec<(String, i64)>,
    chart_data: Vec<(String, i64)>,
    anim_progress: f32, 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab { Today, Week }

#[derive(Debug, Clone)]
pub enum Message {
    SwitchTab(Tab),
    NavPrev,
    NavNext,
    NavToday,
    RefreshData,
    AnimationTick, 
    None,
}

impl FocusdApp {
    pub fn new() -> (Self, Task<Message>) {
        // Run default logic
        let db = Db::init().expect("DB Init Failed");
        let app = Self {
            db,
            config: Config::load(),
            current_tab: Tab::Today,
            ref_date: Local::now().date_naive(),
            total_str: String::from("--"),
            apps: vec![],
            chart_data: vec![],
            anim_progress: 0.0,
        };

        // Trigger loading
        (app, Task::perform(async {}, |_| Message::RefreshData))
    }

    pub fn subscription(&self) -> Subscription<Message> {
        // Run animation timer if animation is incomplete (< 1.0)
        // Works for BOTH Tabs now (Today and Week)
        if self.anim_progress < 1.0 {
            iced::time::every(StdDur::from_millis(16)).map(|_| Message::AnimationTick)
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, msg: Message) -> Task<Message> {
        match msg {
            Message::SwitchTab(tab) => {
                self.current_tab = tab;
                self.ref_date = Local::now().date_naive();
                self.anim_progress = 0.0;
                Task::perform(async {}, |_| Message::RefreshData)
            }
            Message::AnimationTick => {
                // Smooth easing
                self.anim_progress += (1.0 - self.anim_progress) * 0.15;
                if self.anim_progress > 0.999 { self.anim_progress = 1.0; }
                Task::none()
            }
            Message::NavPrev => {
                self.ref_date -= match self.current_tab { Tab::Today => Duration::days(1), Tab::Week => Duration::days(7) };
                self.anim_progress = 0.0;
                Task::perform(async {}, |_| Message::RefreshData)
            }
            Message::NavNext => {
                let now = Local::now().date_naive();
                let next = self.ref_date + match self.current_tab { Tab::Today => Duration::days(1), Tab::Week => Duration::days(7) };
                if next <= now {
                    self.ref_date = next;
                    self.anim_progress = 0.0;
                    Task::perform(async {}, |_| Message::RefreshData)
                } else { Task::none() }
            }
            Message::NavToday => {
                self.ref_date = Local::now().date_naive();
                self.anim_progress = 0.0;
                Task::perform(async {}, |_| Message::RefreshData)
            }
            Message::RefreshData => {
                self.fetch_data();
                Task::none()
            }
            Message::None => Task::none(),
        }
    }

    fn fetch_data(&mut self) {
        let (start, end) = self.get_date_range();

        if let Ok(data) = self.db.get_app_usage_range(start, end) {
            let total: i64 = data.iter().map(|(_, s)| s).sum();
            self.total_str = self.fmt_full(total);
            self.apps = data;
        }

        if self.current_tab == Tab::Week {
            let mut chart = vec![];
            if let Ok(map) = self.db.get_daily_totals(start, end) {
                let mut d = start;
                while d <= end {
                    let val = *map.get(&d.to_string()).unwrap_or(&0);
                    chart.push((d.format("%a").to_string(), val));
                    d += Duration::days(1);
                }
            }
            self.chart_data = chart;
        }
    }

    fn get_date_range(&self) -> (NaiveDate, NaiveDate) {
        match self.current_tab {
            Tab::Today => (self.ref_date, self.ref_date),
            Tab::Week => {
                let w = self.ref_date.weekday();
                let mon = self.ref_date - Duration::days(w.num_days_from_monday() as i64);
                let sun = mon + Duration::days(6);
                (mon, sun)
            }
        }
    }

    fn fmt_full(&self, s: i64) -> String {
        let h = s/3600; let m = (s%3600)/60; let sec = s%60;
        if h > 0 { format!("{:02}h {:02}m {:02}s", h, m, sec) }
        else { format!("{:02}m {:02}s", m, sec) }
    }
    
    // === VIEW ===
    pub fn view(&self) -> Element<'_, Message> {
        let header = self.view_header();
        
        // TOTAL
        let hero = container(column![
             text("Total Focus").size(14).color(C_TEXT),
             text(&self.total_str).size(50).color(C_SKY)
        ].spacing(4).align_x(iced::Alignment::Center)).width(Length::Fill).padding(30).style(card);

        // CHART (Animated)
        let chart_section: Element<Message> = if self.current_tab == Tab::Week {
             let chart_canvas = canvas(AnimatedChart { data: self.chart_data.clone(), progress: self.anim_progress })
                .width(Length::Fill)
                .height(Length::Fixed(180.0));
             container(
                 Element::from(chart_canvas).map(|_| Message::None)
             ).style(card).padding(20).into()
        } else {
             column![].into()
        };

        // LIST (Animated)
        let list = self.view_list();

        let content = column![
            header, 
            rule::Rule::horizontal(1).style(separator), // Works now with 'rule' import
            hero, 
            chart_section, 
            list
        ].spacing(20);

        container(
            scrollable(container(content).padding(20))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| container::Style { background: Some(Background::Color(C_BG)), ..Default::default() })
        .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        let label = if self.current_tab == Tab::Today {
             self.ref_date.format("%A, %b %d").to_string()
        } else {
             let (s, e) = self.get_date_range();
             format!("{} - {}", s.format("%b %d"), e.format("%b %d"))
        };

        let nav = row![
             button(text("Today").size(12)).on_press(Message::NavToday).padding([5,12]).style(ghost_btn),
             horizontal_space().width(10),
             button(text("◄").size(16)).on_press(Message::NavPrev).style(ghost_btn),
             text(label).size(16).color(C_TEXT).width(Length::Fixed(180.0)).center(),
             button(text("►").size(16)).on_press(Message::NavNext).style(ghost_btn),
        ].align_y(iced::Alignment::Center);

        row![
             column![
                 text("Focusd").size(24).color(C_SKY).font(iced::font::Font::MONOSPACE),
             ],
             horizontal_space().width(Length::Fill),
             column![
                 row![
                     button(text("Day").size(14)).on_press(Message::SwitchTab(Tab::Today))
                         .padding([8,20]).style(if self.current_tab == Tab::Today { tab_active } else { tab_inactive }),
                     button(text("Week").size(14)).on_press(Message::SwitchTab(Tab::Week))
                         .padding([8,20]).style(if self.current_tab == Tab::Week { tab_active } else { tab_inactive }),
                 ].spacing(10),
                 vertical_space(10.0),
                 nav
             ].align_x(iced::Alignment::End)
        ].align_y(iced::Alignment::Center).into()
    }

    fn view_list(&self) -> Element<'_, Message> {
         let mut col = column![].spacing(8);
         let max = self.apps.iter().map(|(_,s)| *s).max().unwrap_or(0).max(1) as f32;
         
         let track_width_px = 250.0;

         for (raw, sec) in &self.apps {
              if raw.trim().is_empty() { continue; }
              let name = self.config.alias.get(raw).unwrap_or(raw);
              
              let mut pct = (*sec as f32 / max).clamp(0.0, 1.0);
              // Ensure barely used apps still have visible sliver
              if pct > 0.0 && pct < 0.01 { pct = 0.01; }
              // Animation applied to bar length
              pct *= self.anim_progress;

              let bar_fill_width = track_width_px * pct;

              let row = row![
                  text(name).width(Length::FillPortion(2)).color(C_TEXT).size(15),
                  
                  container(
                      container(text(" ")) 
                        .width(Length::Fixed(bar_fill_width)) 
                        .height(6)
                        .style(|_| container::Style { 
                            background: Some(Background::Color(C_MAYA)), 
                            border: Border { radius: 3.0.into(), ..Default::default() },
                            ..Default::default() 
                        })
                  )
                  .width(Length::Fixed(track_width_px)) 
                  .align_x(iced::Alignment::Start),
                  
                  text(self.fmt_short(*sec)).size(13).color(C_PETAL).width(Length::Fixed(100.0)).align_x(iced::alignment::Horizontal::Right)
              ].spacing(20).align_y(iced::Alignment::Center);
              
              col = col.push(container(row).padding([15, 20]).style(list_item));
         }
         col.into()
    }
    
    fn fmt_short(&self, s: i64) -> String {
        let h = s/3600; let m = (s%3600)/60; let sec = s%60;
        if h > 0 { format!("{}h {}m {}s", h, m, sec) } 
        else if m > 0 { format!("{}m {}s", m, sec) }
        else { format!("{}s", sec) }
    }
}

fn vertical_space(h: f32) -> iced::widget::Space {
    iced::widget::Space::with_height(Length::Fixed(h))
}