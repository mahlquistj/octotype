use std::collections::HashMap;

use derive_more::From;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ParameterDefinitions = HashMap<String, Definition>;

#[derive(Debug, Error)]
pub enum ParameterError {
    #[error("Invalid range: {min} > {max}")]
    InvalidRange { min: usize, max: usize },

    #[error("Invalid step size: {step} > {min}")]
    InvalidStepSize { step: usize, min: usize },

    #[error("Default value is higher than max value: {default} > {max}")]
    DefaultTooHigh { default: usize, max: usize },

    #[error("Default value is lower than min value: {default} > {min}")]
    DefaultTooLow { default: usize, min: usize },

    #[error("Selection is empty")]
    EmptySelection,
}

#[derive(Debug)]
pub struct Parameter {
    pub value: ValueType,
    pub definition: Definition,
}

#[derive(Debug, From)]
pub enum ValueType {
    Number(usize),
    String(String),
    Bool(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Definition {
    Range {
        #[serde(default)]
        min: usize,
        #[serde(default = "default_range_max")]
        max: usize,
        #[serde(default = "default_range_step")]
        step: usize,
        default: Option<usize>,
    },
    Selection {
        options: Vec<String>,
        default: Option<String>,
    },
    Toggle(bool),
    FixedNumber(usize),
    FixedString(String),
}

impl Definition {
    pub fn is_changeable(&self) -> bool {
        !matches!(self, Self::FixedNumber(_) | Self::FixedString(_))
    }

    pub fn into_parameter(self) -> Result<Parameter, ParameterError> {
        self.get_default_value().map(|value| Parameter {
            definition: self,
            value,
        })
    }

    fn get_default_value(&self) -> Result<ValueType, ParameterError> {
        self.evaluate().map(|_| match self {
            Self::Range { min, default, .. } => ValueType::Number(default.unwrap_or(*min)),
            Self::Selection { options, default } => ValueType::String(
                // SAFETY: We should evaluate the definition before accessing default values, or else
                // the below expect would fail in some cases
                default.clone().unwrap_or_else(|| {
                    options
                        .first()
                        .cloned()
                        .expect("No default set for selection")
                }),
            ),
            Self::Toggle(b) => (*b).into(),
            Self::FixedNumber(num) => (*num).into(),
            Self::FixedString(str) => str.clone().into(),
        })
    }

    fn evaluate(&self) -> Result<(), ParameterError> {
        match self {
            Self::Range {
                min,
                max,
                step,
                default,
            } => {
                if min > max {
                    return Err(ParameterError::InvalidRange {
                        min: *min,
                        max: *max,
                    });
                } else if step > max {
                    return Err(ParameterError::InvalidStepSize {
                        step: *step,
                        min: *min,
                    });
                }

                if let Some(value) = default {
                    if value > max {
                        return Err(ParameterError::DefaultTooHigh {
                            default: *value,
                            max: *max,
                        });
                    } else if value < min {
                        return Err(ParameterError::DefaultTooLow {
                            default: *value,
                            min: *min,
                        });
                    }
                }
            }
            Self::Selection { options, .. } => {
                if options.is_empty() {
                    return Err(ParameterError::EmptySelection);
                }
            }
            _ => (),
        }

        Ok(())
    }
}

pub const fn default_range_step() -> usize {
    1
}

pub const fn default_range_max() -> usize {
    usize::MAX
}
