use std::collections::{BTreeMap, HashMap};

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Span, ToSpan},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, Padding, Paragraph,
    },
    Frame,
};

use crate::utils::{Timestamp, ROUNDED_BLOCK};

use super::Segment;

#[derive(Default, Debug, Clone, Copy)]
pub struct Wpm {
    pub(crate) raw: f64,
    pub(crate) actual: f64,
}

impl Wpm {
    fn min_max(&self) -> [f64; 2] {
        if self.raw > self.actual {
            return [self.actual, self.raw];
        }

        [self.raw, self.actual]
    }
}

#[derive(Default, Debug)]
pub struct RunningStats {
    points: Vec<GraphPoint>,
    deletetions: u16,
    y_bounds: [f64; 2],
}

impl RunningStats {
    pub fn update(&mut self, point: GraphPoint, delete: bool) {
        let [min, max] = point.wpm.min_max();
        if min < self.y_bounds[0] {
            self.y_bounds[0] = min;
        }

        if max > self.y_bounds[1] {
            self.y_bounds[1] = max;
        }

        self.points.push(point);

        if delete {
            self.deletetions += 1;
        }
    }

    pub fn build_stats(&self, text: &[Segment]) -> Stats {
        let (final_wpm, final_acc) = self
            .points
            .last()
            .map(|gp| (gp.wpm, gp.acc * 100.0))
            .unwrap_or_default();
        let errors_count = text.iter().map(Segment::actual_errors).sum();
        let corrected = text
            .iter()
            .map(Segment::current_errors)
            .sum::<u16>()
            .abs_diff(errors_count);

        let mut character_collection = HashMap::<char, u16>::new();

        self.points.iter().for_each(|gp| {
            if let Some(error) = gp.error {
                character_collection
                    .entry(error)
                    .and_modify(|count| *count += 1)
                    .or_insert_with(|| 1);
            }
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

        let (raw_wpm, actual_wpm, errors, acc) = self
            .points
            .iter()
            .map(|gp| {
                (
                    (gp.time, gp.wpm.raw),
                    (gp.time, gp.wpm.actual),
                    (gp.time, gp.error),
                    (gp.time, gp.acc),
                )
            })
            .collect::<(Vec<_>, Vec<_>, Vec<(Timestamp, Option<char>)>, Vec<_>)>();

        let errors = errors
            .iter()
            .filter_map(|(ts, character)| character.map(|_| (*ts, 0.5)))
            .collect();

        let consistency = coefficient_of_variation(&raw_wpm);

        Stats {
            characters,
            raw_wpm,
            actual_wpm,
            errors,
            acc,
            deletions: self.deletetions,
            errors_count,
            corrected,
            final_wpm,
            final_acc,
            y_bounds: self.y_bounds,
            x_bounds: [
                self.points.first().expect("No data").time,
                self.points.last().expect("No data").time,
            ],
            consistency,
        }
    }
}

fn coefficient_of_variation(data: &[(f64, f64)]) -> f64 {
    let values: Vec<f64> = data.iter().map(|&(_, v)| v).collect();
    let mean: f64 = values.iter().sum::<f64>() / values.len() as f64;

    if mean == 0.0 {
        return 0.0;
    }

    let variance: f64 =
        values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;

    let std_dev = variance.sqrt();
    (std_dev / mean) * 100.0
}

#[derive(Debug, Clone, Copy)]
pub struct GraphPoint {
    pub(crate) time: Timestamp,
    pub(crate) wpm: Wpm,
    pub(crate) error: Option<char>,
    pub(crate) acc: f64,
}

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
}

impl Stats {
    pub fn render(&self, frame: &mut Frame, area: Rect) -> std::io::Result<()> {
        let [text, charts] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(area);

        let [wpm, accuracy] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(charts);

        let text_area = Block::new().padding(Padding::right(1)).inner(text);

        let [summary, characters] =
            Layout::vertical([Constraint::Length(9), Constraint::Fill(1)]).areas(text_area);

        let raw_wpm = Dataset::default()
            .name("Raw Wpm")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Gray))
            .data(&self.raw_wpm);

        let actual_wpm = Dataset::default()
            .name("Wpm")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&self.actual_wpm);

        let errors = Dataset::default()
            .name("Errors")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(Color::Red))
            .data(&self.errors);

        let acc = Dataset::default()
            .name("Accuracy")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .data(&self.acc);

        let wpm_chart = Chart::new(vec![raw_wpm, actual_wpm])
            .block(ROUNDED_BLOCK.title("Words/min".to_span().bold()))
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .labels([Span::raw("start"), Span::raw("end")])
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
                    .bounds(self.x_bounds),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels([Span::raw("0%"), Span::raw("50%"), Span::raw("100%")])
                    .bounds([0.0, 1.0]),
            )
            .legend_position(Some(LegendPosition::BottomRight));

        frame.render_widget(accuracy_chart, accuracy);

        let summary_text = Paragraph::new(vec![
            Line::from(format!("Wpm (Actual): {:.2}", self.final_wpm.actual)),
            Line::from(format!("Wpm (Raw)   : {:.2}", self.final_wpm.raw)),
            Line::from(format!("Accuracy    : {}%", self.final_acc.trunc())),
            Line::from(format!("consistency : {}%", self.consistency.trunc())),
            Line::from(format!("Deletions   : {}", self.deletions)),
            Line::from(format!("Errors      : {}", self.errors_count)),
            Line::from(format!("Corrections : {}", self.corrected)),
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
                            c.to_span().style(Style::new().add_modifier(Modifier::BOLD)),
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

        Ok(())
    }
}
