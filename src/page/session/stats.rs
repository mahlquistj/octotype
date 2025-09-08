use super::Segment;
use std::collections::{BTreeMap, HashMap};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, Padding, Paragraph,
    },
};

use crate::{
    app::Message,
    config::Config,
    page,
    utils::{ROUNDED_BLOCK, Timestamp},
};

/// A struct describing words pr. min
#[derive(Default, Debug, Clone, Copy)]
pub struct Wpm {
    pub(crate) raw: f64,
    pub(crate) actual: f64,
}

impl Wpm {
    /// Returns the `[minimum, maximum]` values by comparing `self.raw` and `self.actual`
    fn min_max(&self) -> [f64; 2] {
        if self.raw > self.actual {
            return [self.actual, self.raw];
        }

        [self.raw, self.actual]
    }
}

/// A collection of statistics from a running session
#[derive(Default, Debug)]
pub struct RunningStats {
    /// The errors that occurred during the session (Not respecting corrections)
    errors: Vec<(Timestamp, char)>,

    /// The accuracy points collected
    acc: Vec<(Timestamp, f64)>,

    /// Wpm points collected
    wpm: Vec<(Timestamp, Wpm)>,

    /// How many times a deletion occurred
    deletions: u16,

    /// Y-axis bounds for wpm
    y_bounds: [f64; 2],
}

impl RunningStats {
    /// Updates the `RunningStats`
    pub fn update(
        &mut self,
        time: Timestamp,
        acc: f64,
        wpm: Option<Wpm>,
        error: Option<char>,
        delete: bool,
    ) {
        if let Some(w) = wpm {
            let [min, max] = w.min_max();
            if min < self.y_bounds[0] {
                self.y_bounds[0] = min;
            }

            if max > self.y_bounds[1] {
                self.y_bounds[1] = max;
            }

            self.wpm.push((time, w));
        }

        if let Some(e) = error {
            self.errors.push((time, e));
        }

        self.acc.push((time, acc));

        if delete {
            self.deletions += 1;
        }
    }

    /// Builds the `RunningStats` into a `Stats` page
    pub fn build_stats(
        self,
        text: &[Segment],
        final_wpm: Wpm,
        final_acc: f64,
        time: Timestamp,
    ) -> Stats {
        let Self {
            errors,
            acc,
            wpm,
            deletions,
            y_bounds,
        } = self;

        let errors_count = text.iter().map(Segment::actual_errors).sum();
        let corrected = text
            .iter()
            .map(Segment::current_errors)
            .sum::<u16>()
            .abs_diff(errors_count);

        let mut character_collection = HashMap::<char, u16>::new();

        errors.iter().for_each(|(_, char)| {
            character_collection
                .entry(*char)
                .and_modify(|count| *count += 1)
                .or_insert_with(|| 1);
        });

        let mut characters = BTreeMap::new();

        character_collection
            .into_iter()
            .for_each(|(character, count)| {
                characters
                    .entry(count)
                    .and_modify(|chars: &mut Vec<char>| chars.push(character))
                    .or_insert_with(|| vec![character]);
            });

        let (raw_wpm, actual_wpm) = wpm
            .iter()
            .copied()
            .map(|(time, wpm)| ((time, wpm.raw), (time, wpm.actual)))
            .collect::<(Vec<(f64, f64)>, Vec<(f64, f64)>)>();

        let errors = errors.iter().map(|(ts, _)| (*ts, 0.5)).collect();

        let consistency = coefficient_of_variation(&raw_wpm);

        Stats {
            characters,
            raw_wpm,
            actual_wpm,
            errors,
            acc,
            deletions,
            errors_count,
            corrected,
            final_wpm,
            final_acc,
            y_bounds,
            x_bounds: [
                wpm.first().expect("No data").0,
                wpm.last().expect("No data").0,
            ],
            consistency,
            time,
        }
    }
}

/// Calculates the coeffectient of variation used for calculating consistency
fn coefficient_of_variation(data: &[(f64, f64)]) -> f64 {
    let values: Vec<f64> = data.iter().map(|&(_, v)| v).collect();
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

    if mean == 0.0 {
        return 0.0;
    }

    let variance: f64 =
        values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

    let std_dev = variance.sqrt();
    let res = (std_dev / mean).mul_add(-100.0, 100.0);

    if res.is_finite() {
        return res.abs();
    }

    0.0
}

/// Page: Stats
///
/// Contains data and logic to show statistics after a session.
///
#[derive(Debug, Clone)]
pub struct Stats {
    characters: BTreeMap<u16, Vec<char>>,
    raw_wpm: Vec<(Timestamp, f64)>,
    actual_wpm: Vec<(Timestamp, f64)>,
    errors: Vec<(Timestamp, f64)>,
    acc: Vec<(Timestamp, f64)>,
    deletions: u16,
    errors_count: u16,
    corrected: u16,
    final_wpm: Wpm,
    final_acc: f64,
    y_bounds: [f64; 2],
    x_bounds: [f64; 2],
    consistency: f64,
    time: f64,
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

        let theme = &config.theme.plot;

        let raw_wpm = Dataset::default()
            .name("Raw Wpm")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.raw_wpm))
            .data(&self.raw_wpm);

        let actual_wpm = Dataset::default()
            .name("Wpm")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.actual_wpm))
            .data(&self.actual_wpm);

        let errors = Dataset::default()
            .name("Errors")
            .marker(theme.scatter_symbol.as_marker())
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(theme.errors))
            .data(&self.errors);

        let acc = Dataset::default()
            .name("Accuracy")
            .marker(theme.line_symbol.as_marker())
            .graph_type(GraphType::Line)
            .style(Style::default().fg(theme.accuracy))
            .data(&self.acc);

        let wpm_chart = Chart::new(vec![raw_wpm, actual_wpm])
            .block(ROUNDED_BLOCK.title("Words/min".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .labels([
                        Span::raw(self.x_bounds[0].trunc().to_string()),
                        Span::raw(self.x_bounds[1].trunc().to_string()),
                    ])
                    .bounds(self.x_bounds),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels([
                        Span::raw(self.y_bounds[0].trunc().to_string()),
                        Span::raw((self.y_bounds[1] / 2.0).trunc().to_string()),
                        Span::raw((self.y_bounds[1]).trunc().to_string()),
                    ])
                    .bounds(self.y_bounds),
            )
            .legend_position(Some(LegendPosition::BottomRight));

        frame.render_widget(wpm_chart, wpm);

        let accuracy_chart = Chart::new(vec![acc, errors])
            .block(ROUNDED_BLOCK.title("Accuracy".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .labels([Span::raw("start"), Span::raw("end")])
                    .bounds([self.x_bounds[0], self.time]),
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
            Line::from(format!("Time (Minutes): {:.2}", self.time)),
            Line::from(format!("Wpm (Actual)  : {:.2}", self.final_wpm.actual)),
            Line::from(format!("Wpm (Raw)     : {:.2}", self.final_wpm.raw)),
            Line::from(format!("Accuracy      : {}%", self.final_acc.trunc())),
            Line::from(format!("Consistency   : {}%", self.consistency.trunc())),
            Line::from(format!("Deletions     : {}", self.deletions)),
            Line::from(format!("Errors        : {}", self.errors_count)),
            Line::from(format!("Corrections   : {}", self.corrected)),
            Line::from(format!("Polls         : {}", self.raw_wpm.len())),
        ])
        .block(
            ROUNDED_BLOCK
                .borders(Borders::TOP)
                .title("Summary".to_span().bold()),
        );

        frame.render_widget(summary_text, summary);

        let character_lines: Vec<Line> = self
            .characters
            .iter()
            .rev()
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

    pub fn render_top(&self, _config: &Config) -> Option<Line> {
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
            return Some(Message::Reset); // TODO: Return to menu - need to pass mode configs
        }

        None
    }
}
