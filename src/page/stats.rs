use std::collections::BTreeMap;

use crossterm::event::{Event, KeyCode};
use gladius::{CharacterResult, statistics::Statistics};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, Padding, Paragraph,
    },
};

use crate::{app::Message, config::Config, utils::ROUNDED_BLOCK};

type PlotData = Vec<(f64, f64)>;

/// Page: Stats
///
/// Contains data and logic to show statistics after a session.
///
#[derive(Debug, Clone)]
pub struct Stats {
    gladius_stats: Statistics,
    datasets: DataSets,
    wpm_low: f64,
    wpm_high: f64,
    char_errors: BTreeMap<usize, Vec<char>>,
}

#[derive(Debug, Clone)]
pub struct DataSets {
    errors: PlotData,
    raw_wpm: PlotData,
    actual_wpm: PlotData,
    raw_accuracy: PlotData,
    actual_accuracy: PlotData,
    consistency: PlotData,
}

impl From<Statistics> for Stats {
    fn from(value: Statistics) -> Self {
        let measurements_len = value.measurements.len();
        let mut raw_wpm = Vec::with_capacity(measurements_len);
        let mut actual_wpm = Vec::with_capacity(measurements_len);
        let mut raw_accuracy = Vec::with_capacity(measurements_len);
        let mut actual_accuracy = Vec::with_capacity(measurements_len);
        let mut consistency = Vec::with_capacity(measurements_len);
        let mut errors = Vec::with_capacity(value.counters.errors);
        let mut wpm_low = f64::MAX;
        let mut wpm_high = f64::MIN;

        value.measurements.iter().for_each(|m| {
            raw_wpm.push((m.timestamp, m.wpm.raw));
            actual_wpm.push((m.timestamp, m.wpm.actual));
            raw_accuracy.push((m.timestamp, m.accuracy.raw));
            actual_accuracy.push((m.timestamp, m.accuracy.actual));
            consistency.push((m.timestamp, m.consistency.actual_percent));

            wpm_low = wpm_low.min(m.wpm.raw.min(m.wpm.actual));
            wpm_high = wpm_high.max(m.wpm.raw.max(m.wpm.actual));
        });

        value.input_history.iter().for_each(|input| {
            if input.result == CharacterResult::Wrong {
                errors.push((input.timestamp, 1.0));
            }
        });

        let datasets = DataSets {
            errors,
            raw_wpm,
            actual_wpm,
            raw_accuracy,
            actual_accuracy,
            consistency,
        };

        let mut char_errors = BTreeMap::new();
        value
            .counters
            .char_errors
            .iter()
            .for_each(|(character, count)| {
                char_errors
                    .entry(*count)
                    .and_modify(|chars: &mut Vec<char>| chars.push(*character))
                    .or_insert_with(|| vec![*character]);
            });

        Self {
            gladius_stats: value,
            datasets,
            wpm_low,
            wpm_high,
            char_errors,
        }
    }
}

// Rendering logic
impl Stats {
    pub fn render(&self, frame: &mut Frame, area: Rect, config: &Config) {
        let [text, charts] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);

        let [wpm, accuracy] =
            Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)])
                .areas(charts);

        let text_area = Block::new().padding(Padding::right(1)).inner(text);

        let [summary, characters] =
            Layout::vertical([Constraint::Length(10), Constraint::Fill(1)]).areas(text_area);

        let theme = &config.settings.theme.plot;

        let total_duration = self.gladius_stats.duration.as_secs_f64();

        let raw_wpm = Dataset::default()
            .name("Raw Wpm")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.raw_wpm))
            .data(&self.datasets.raw_wpm);

        let actual_wpm = Dataset::default()
            .name("Wpm")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.actual_wpm))
            .data(&self.datasets.actual_wpm);

        let errors = Dataset::default()
            .name("Errors")
            .marker(theme.scatter_symbol.as_marker())
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(theme.errors))
            .data(&self.datasets.errors);

        let raw_accuracy = Dataset::default()
            .name("Raw Accuracy")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.actual_wpm))
            .data(&self.datasets.raw_accuracy);

        let actual_accuracy = Dataset::default()
            .name("Accuracy")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.accuracy))
            .data(&self.datasets.actual_accuracy);

        let consistency = Dataset::default()
            .name("Consistency")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Blue))
            .data(&self.datasets.consistency);

        let wpm_chart = Chart::new(vec![raw_wpm, actual_wpm])
            .block(ROUNDED_BLOCK.title("Words/min".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .labels([
                        Span::raw(0.0f64.trunc().to_string()),
                        Span::raw(total_duration.trunc().to_string()),
                    ])
                    .bounds([0.0, total_duration]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels([
                        Span::raw(self.wpm_low.trunc().to_string()),
                        Span::raw(((self.wpm_high + self.wpm_low) / 2.0).trunc().to_string()),
                        Span::raw((self.wpm_high).trunc().to_string()),
                    ])
                    .bounds([self.wpm_low, self.wpm_high]),
            )
            .legend_position(Some(LegendPosition::BottomRight));

        frame.render_widget(wpm_chart, wpm);

        let accuracy_chart = Chart::new(vec![consistency, raw_accuracy, actual_accuracy, errors])
            .block(ROUNDED_BLOCK.title("Accuracy".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .labels([Span::raw("start"), Span::raw("end")])
                    .bounds([0.0, total_duration]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels([Span::raw("0%"), Span::raw("50%"), Span::raw("100%")])
                    .bounds([0.0, 100.0]),
            )
            .legend_position(Some(LegendPosition::BottomRight));

        frame.render_widget(accuracy_chart, accuracy);

        let summary_text = Paragraph::new(vec![
            Line::from(format!("Time (Minutes): {:.2}", total_duration / 60.0)),
            Line::from(format!(
                "Wpm (Actual)  : {:.2}",
                self.gladius_stats.wpm.actual
            )),
            Line::from(format!("Wpm (Raw)     : {:.2}", self.gladius_stats.wpm.raw)),
            Line::from(format!(
                "Accuracy      : {}%",
                self.gladius_stats.accuracy.actual.trunc()
            )),
            Line::from(format!(
                "Consistency   : {}%",
                self.gladius_stats.consistency.actual_percent.trunc()
            )),
            Line::from(format!(
                "Deletions     : {} ({} wrong)",
                self.gladius_stats.counters.deletes, self.gladius_stats.counters.wrong_deletes
            )),
            Line::from(format!(
                "Errors        : {}",
                self.gladius_stats.counters.errors
            )),
            Line::from(format!(
                "Corrections   : {}",
                self.gladius_stats.counters.corrections
            )),
        ])
        .block(
            ROUNDED_BLOCK
                .borders(Borders::TOP)
                .title("Summary".to_span().bold()),
        );

        frame.render_widget(summary_text, summary);

        let character_lines: Vec<Line> = self
            .char_errors
            .iter()
            .flat_map(|(fails, chars)| {
                chars
                    .iter()
                    .map(|c| {
                        Line::default().spans(vec![
                            c.to_span().style(Style::new().bold()),
                            Span::from(format!(": {fails}")),
                        ])
                    })
                    .collect::<Vec<Line>>()
            })
            .collect();

        let character_errors = Paragraph::new(character_lines).block(
            ROUNDED_BLOCK
                .borders(Borders::TOP)
                .title("Failed characters".to_span().bold()),
        );

        frame.render_widget(character_errors, characters);
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line<'_>> {
        Some(Line::raw("<Enter> to go back to the menu"))
    }

    pub fn handle_events(
        &self,
        event: &crossterm::event::Event,
        _config: &Config,
    ) -> Option<Message> {
        if let Event::Key(key) = event
            && key.code == KeyCode::Enter
        {
            return Some(Message::Reset);
        }

        None
    }
}
