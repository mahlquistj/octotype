use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Axis, Block, Chart, Dataset, GraphType, LegendPosition, List, Paragraph},
};
use web_time::SystemTime;

use crate::{
    app::Message,
    config::Config,
    statistics::SessionStatistics,
    utils::{ROUNDED_BLOCK, center},
};

/// Page: History
///
/// Shows saved statistics history and improvements over time.
#[derive(Debug)]
pub struct History {
    sessions: Vec<SessionStatistics>,
    selected_index: usize,
    view_mode: ViewMode,
}

#[derive(Debug, Clone, Copy)]
enum ViewMode {
    List,
    Trends,
}

impl History {
    pub fn new(config: &Config) -> Result<Self, String> {
        let sessions = if let Some(stats_manager) = &config.statistics_manager {
            stats_manager
                .load_all_sessions()
                .map_err(|e| e.to_string())?
        } else {
            Vec::new()
        };

        Ok(Self {
            sessions,
            selected_index: 0,
            view_mode: ViewMode::List,
        })
    }

    fn get_selected_session(&self) -> Option<&SessionStatistics> {
        self.sessions.get(self.selected_index)
    }

    const fn move_selection_up(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        self.selected_index = if self.selected_index == 0 {
            self.sessions.len() - 1
        } else {
            self.selected_index - 1
        };
    }

    const fn move_selection_down(&mut self) {
        if self.sessions.is_empty() {
            return;
        }
        self.selected_index = (self.selected_index + 1) % self.sessions.len();
    }

    fn format_timestamp(timestamp: SystemTime) -> String {
        let now = SystemTime::now();
        let duration = now.duration_since(timestamp).unwrap_or_default();
        let secs = duration.as_secs();
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;

        if days > 0 {
            format!("{}d {}h {}m ago", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m ago", hours, minutes)
        } else if minutes > 0 {
            format!("{}m ago", minutes)
        } else {
            "just now".to_string()
        }
    }

    fn render_list_view(&self, frame: &mut Frame, area: Rect, config: &Config) {
        if self.sessions.is_empty() {
            let no_data = Paragraph::new(
                "No statistics saved yet.\nComplete a typing session to see your history here.",
            )
            .block(ROUNDED_BLOCK.title("Statistics History".to_span().bold()))
            .centered();
            frame.render_widget(no_data, area);
            return;
        }

        let [detail_area, list_area] =
            Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(area);

        // Render session list
        let items = self.sessions.iter().enumerate().map(|(i, session)| {
            let mut selector = "  ";
            let style = if i == self.selected_index {
                selector = "> ";
                Style::new()
                    .fg(config.settings.theme.text.highlight)
                    .reversed()
            } else {
                Style::new()
            };

            let mode_source = format!(
                "{} / {}",
                session.session_config.mode_name, session.session_config.source_name
            );
            let wpm = format!("{:.1} wpm", session.statistics.wpm_actual);
            let accuracy = format!("{:.0}%", session.statistics.accuracy_actual);
            let time_ago = Self::format_timestamp(session.timestamp);

            Line::from(vec![
                Span::raw(selector),
                Span::styled(
                    format!(
                        "{:<20} | {:<8} | {:<5} | {}",
                        mode_source, wpm, accuracy, time_ago
                    ),
                    style,
                ),
            ])
        });

        let list = List::new(items).block(ROUNDED_BLOCK.title("Session History".to_span().bold()));
        frame.render_widget(list, list_area);

        // Render selected session details
        if let Some(session) = self.get_selected_session() {
            let settings = vec![
                Line::from(format!("Mode: {}", session.session_config.mode_name)),
                Line::from(format!("Source: {}", session.session_config.source_name)),
                Line::from(format!(
                    "Deletions: {}",
                    if session.session_config.allow_deletions {
                        "Allowed"
                    } else {
                        "Disabled"
                    }
                )),
                Line::from(format!(
                    "Errors: {}",
                    if session.session_config.allow_errors {
                        "Allowed"
                    } else {
                        "Disabled"
                    }
                )),
                session.session_config.time_limit.map_or_else(
                    || Line::from("Time Limit: None"),
                    |limit| Line::from(format!("Time Limit: {:.0}s", limit)),
                ),
                session.session_config.words_typed_limit.map_or_else(
                    || Line::from("Word Limit: None"),
                    |limit| Line::from(format!("Word Limit: {}", limit)),
                ),
            ];
            let stats = vec![
                Line::from(format!(
                    "Time: {:.2} min",
                    session.statistics.duration / 60.0
                )),
                Line::from(format!(
                    "WPM (Actual): {:.2}",
                    session.statistics.wpm_actual
                )),
                Line::from(format!("WPM (Raw): {:.2}", session.statistics.wpm_raw)),
                Line::from(format!(
                    "Accuracy: {:.1}%",
                    session.statistics.accuracy_actual
                )),
                Line::from(format!(
                    "Consistency: {:.1}%",
                    session.statistics.consistency_actual_percent
                )),
                Line::from(format!("Errors: {}", session.statistics.errors)),
                Line::from(format!("Corrections: {}", session.statistics.corrections)),
                Line::from(format!(
                    "Correct Characters: {}",
                    session.statistics.corrects
                )),
                Line::from(format!("Total Added: {}", session.statistics.adds)),
            ];

            let outer_block = ROUNDED_BLOCK.title("Session Details".to_span().bold());
            let inner_area = outer_block.inner(detail_area);

            let [settings_area, stats_area] =
                Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .areas(inner_area);

            frame.render_widget(outer_block, detail_area);
            frame.render_widget(
                Paragraph::new(settings)
                    .block(Block::new().title(Span::from("Settings").bold().underlined())),
                settings_area,
            );
            frame.render_widget(
                Paragraph::new(stats)
                    .block(Block::new().title(Span::from("Stats").bold().underlined())),
                stats_area,
            );
        }
    }

    fn render_trends_view(&self, frame: &mut Frame, area: Rect, config: &Config) {
        if self.sessions.len() < 2 {
            let no_data = Paragraph::new("Need at least 2 sessions to show trends.\nComplete more typing sessions to see your progress.")
                .block(ROUNDED_BLOCK.title("Trends".to_span().bold()))
                .centered();
            frame.render_widget(no_data, area);
            return;
        }

        let [wpm_area, accuracy_area] =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

        // Prepare data for charts - reverse to show chronological order
        let mut wpm_data = Vec::new();
        let mut accuracy_data = Vec::new();

        let sessions_reversed: Vec<_> = self.sessions.iter().rev().collect();

        for (i, session) in sessions_reversed.iter().enumerate() {
            let x = i as f64;
            wpm_data.push((x, session.statistics.wpm_actual));
            accuracy_data.push((x, session.statistics.accuracy_actual));
        }

        let theme = &config.settings.theme.plot;

        // WPM trend chart
        let wpm_dataset = Dataset::default()
            .name("WPM")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.actual_wpm))
            .data(&wpm_data);

        let (wpm_min, wpm_max) = wpm_data
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |acc, (_, y)| {
                (acc.0.min(*y), acc.1.max(*y))
            });

        let wpm_bounds = if wpm_min.is_finite() && wpm_max.is_finite() {
            [wpm_min - 5.0, wpm_max + 5.0]
        } else {
            [0.0, 100.0]
        };

        let wpm_chart = Chart::new(vec![wpm_dataset])
            .block(ROUNDED_BLOCK.title("WPM Progress".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Sessions")
                    .style(Style::default().fg(Color::Gray))
                    .labels((1..=self.sessions.len()).map(|i| i.to_string()))
                    .bounds([0.0, (sessions_reversed.len() - 1) as f64]),
            )
            .y_axis(
                Axis::default()
                    .title("WPM")
                    .style(Style::default().fg(Color::Gray))
                    .labels((wpm_min as usize..=wpm_max as usize).map(|wpm| wpm.to_string()))
                    .bounds(wpm_bounds),
            )
            .legend_position(Some(LegendPosition::BottomLeft));

        frame.render_widget(wpm_chart, wpm_area);

        // Accuracy trend chart
        let accuracy_dataset = Dataset::default()
            .name("Accuracy")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.accuracy))
            .data(&accuracy_data);

        let accuracy_chart = Chart::new(vec![accuracy_dataset])
            .block(ROUNDED_BLOCK.title("Accuracy Progress".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Sessions")
                    .style(Style::default().fg(Color::Gray))
                    .labels((1..=self.sessions.len()).map(|i| i.to_string()))
                    .bounds([0.0, (sessions_reversed.len() - 1) as f64]),
            )
            .y_axis(
                Axis::default()
                    .title("Accuracy (%)")
                    .style(Style::default().fg(Color::Gray))
                    .labels(["0%", "50%", "100%"])
                    .bounds([0.0, 100.0]),
            )
            .legend_position(Some(LegendPosition::BottomLeft));

        frame.render_widget(accuracy_chart, accuracy_area);
    }
}

// Rendering logic
impl History {
    pub fn render(&self, frame: &mut Frame, area: Rect, config: &Config) {
        let area = center(area, Constraint::Percentage(90), Constraint::Percentage(90));

        match self.view_mode {
            ViewMode::List => self.render_list_view(frame, area, config),
            ViewMode::Trends => self.render_trends_view(frame, area, config),
        }
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line<'_>> {
        match self.view_mode {
            ViewMode::List => Some(Line::raw(
                "<Enter> menu | <Tab> trends | <Up/Down> navigate",
            )),
            ViewMode::Trends => Some(Line::raw("<Enter> menu | <Tab> list view")),
        }
    }

    pub fn handle_events(&mut self, event: &Event, _config: &Config) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            match key.code {
                KeyCode::Enter => return Some(Message::Reset),
                KeyCode::Tab => {
                    self.view_mode = match self.view_mode {
                        ViewMode::List => ViewMode::Trends,
                        ViewMode::Trends => ViewMode::List,
                    };
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if matches!(self.view_mode, ViewMode::List) {
                        self.move_selection_up();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if matches!(self.view_mode, ViewMode::List) {
                        self.move_selection_down();
                    }
                }
                _ => (),
            }
        }

        None
    }
}
