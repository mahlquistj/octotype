use crate::modes::{ConditionValues, ModeManager, ResolvedModeConfig};
use crate::sources::ParameterValues;
use crate::page::session::{Segment, TypingSession};
use crate::sources::{SourceError, SourceManager};
use thiserror::Error;

pub struct SessionFactory {
    source_manager: SourceManager,
    mode_manager: ModeManager,
}

impl SessionFactory {
    pub const fn new(source_manager: SourceManager, mode_manager: ModeManager) -> Self {
        Self {
            source_manager,
            mode_manager,
        }
    }

    pub fn create_session(
        &self,
        mode_name: &str,
        parameter_values: Option<ParameterValues>,
        condition_values: Option<ConditionValues>,
    ) -> Result<TypingSession, SessionCreationError> {
        // Get mode configuration
        let mode_config = self
            .mode_manager
            .get_mode(mode_name)
            .ok_or_else(|| SessionCreationError::ModeNotFound(mode_name.to_string()))?;

        // Use provided values or create defaults
        let (param_values, cond_values) =
            if let (Some(p), Some(c)) = (parameter_values, condition_values) {
                (p, c)
            } else {
                self.mode_manager
                    .create_default_values(mode_name)
                    .ok_or_else(|| SessionCreationError::InvalidMode(mode_name.to_string()))?
            };

        // For now, use default source selection (first available source)
        let available_sources = self.source_manager.list_sources();
        let default_source_name = available_sources
            .first()
            .ok_or_else(|| SessionCreationError::InvalidMode("No sources available".to_string()))?
            .to_string();
            
        let source = self
            .source_manager
            .get_source(&default_source_name)
            .ok_or_else(|| SessionCreationError::SourceNotFound(default_source_name.clone()))?;

        // Apply mode parameter overrides to source
        let source_overrides = mode_config.resolve_source_overrides(&default_source_name, &param_values);
        let mut effective_source_params = source.create_default_parameters();
        
        // Override with mode-specified parameters
        // TODO: Implement parameter merging logic
        
        // Create resolved mode config
        let resolved_mode = ResolvedModeConfig::new(
            mode_config.name.clone(),
            param_values,
            cond_values,
            default_source_name.clone(),
            effective_source_params.clone(),
        );

        // Get words from source
        let words = source.fetch(&effective_source_params)?;

        // Convert words to segments
        let segments = self.words_to_segments(words);

        // For now, create a simple session (will be enhanced later for mode support)
        TypingSession::new(segments)
            .map_err(SessionCreationError::SessionCreation)
    }

    pub fn create_default_session(&self) -> Result<TypingSession, SessionCreationError> {
        // Fallback words for when no modes are configured
        let words = vec![
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
            "jumps".to_string(),
            "over".to_string(),
            "lazy".to_string(),
            "dog".to_string(),
            "pack".to_string(),
            "my".to_string(),
            "box".to_string(),
            "with".to_string(),
            "five".to_string(),
            "dozen".to_string(),
            "liquor".to_string(),
            "jugs".to_string(),
        ];

        let segments = self.words_to_segments(words);
        TypingSession::new(segments).map_err(SessionCreationError::SessionCreation)
    }

    fn words_to_segments(&self, words: Vec<String>) -> Vec<Segment> {
        words
            .chunks(5)
            .map(|chunk| {
                let line = chunk
                    .iter()
                    .map(|word| format!("{} ", word))
                    .collect::<String>();
                Segment::from_iter(line.chars())
            })
            .collect()
    }

    pub const fn get_source_manager(&self) -> &SourceManager {
        &self.source_manager
    }

    pub const fn get_mode_manager(&self) -> &ModeManager {
        &self.mode_manager
    }
}

#[derive(Debug, Error)]
pub enum SessionCreationError {
    #[error("Mode '{0}' not found")]
    ModeNotFound(String),

    #[error("Source '{0}' not found")]
    SourceNotFound(String),

    #[error("Invalid mode configuration: {0}")]
    InvalidMode(String),

    #[error("Source error: {0}")]
    Source(#[from] SourceError),

    #[error("Session creation failed: {0}")]
    SessionCreation(#[from] crate::page::session::EmptySessionError),
}
