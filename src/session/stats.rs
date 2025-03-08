use std::{
    collections::{BTreeMap, HashMap},
    error,
};

use ratatui::{
    layout::Rect,
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::utils::Timestamp;

use super::Segment;

#[derive(Debug)]
pub struct TimedData<T> {
    time: Timestamp,
    data: T,
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Wpm {
    pub(crate) raw: f32,
    pub(crate) actual: f32,
}

#[derive(Default, Debug)]
pub struct RunningStats {
    points: Vec<GraphPoint>,
    deletetions: u16,
}

impl RunningStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, time: Timestamp, point: GraphPoint, delete: bool) {
        self.points.push(point);

        if delete {
            self.deletetions += 1;
        }
    }

    pub fn build_stats(&self, text: &Vec<Segment>) -> Stats {
        let (final_wpm, final_acc) = self
            .points
            .last()
            .map(|gp| (gp.wpm, gp.acc))
            .unwrap_or_default();
        let errors = text.iter().map(Segment::actual_errors).sum();
        let corrected = text
            .iter()
            .map(Segment::current_errors)
            .sum::<u16>()
            .abs_diff(errors);

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

        Stats {
            characters,
            graph_data: self.points.clone(),
            deletions: self.deletetions,
            errors,
            corrected,
            final_wpm,
            final_acc,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct GraphPoint {
    pub(crate) time: Timestamp,
    pub(crate) wpm: Wpm,
    pub(crate) error: Option<char>,
    pub(crate) acc: f32,
}

#[derive(Debug)]
pub struct Stats {
    characters: BTreeMap<u16, Vec<char>>,
    graph_data: Vec<GraphPoint>,
    deletions: u16,
    errors: u16,
    corrected: u16,
    final_wpm: Wpm,
    final_acc: f32,
}

impl Stats {
    pub fn render(&self, frame: &mut Frame, area: Rect) -> std::io::Result<()> {
        let stats = Paragraph::new(format!("{self:?}")).wrap(Wrap { trim: false });
        frame.render_widget(stats, area);
        Ok(())
    }
}
