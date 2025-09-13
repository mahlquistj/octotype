use std::collections::HashMap;

use derive_more::Display;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ParameterDefinitions = HashMap<String, Definition>;
pub type ParameterValues = HashMap<String, Parameter>;

#[derive(Debug, Error)]
pub enum ParameterError {
    #[error("Invalid range: {min} > {max}")]
    InvalidRange { min: i64, max: i64 },

    #[error("Invalid step size: {step} > {min}")]
    InvalidStepSize { step: i64, min: i64 },

    #[error("Default value is higher than max value: {default} > {max}")]
    DefaultTooHigh { default: i64, max: i64 },

    #[error("Default value is lower than min value: {default} > {min}")]
    DefaultTooLow { default: i64, min: i64 },

    #[error("Selection is empty")]
    EmptySelection,

    #[error("Default doesn't exist in selection")]
    DefaultNonExistant,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    definition: Definition,
    mutable: bool,
}

impl Parameter {
    pub const fn is_mutable(&self) -> bool {
        if self.mutable {
            self.definition.is_mutable()
        } else {
            self.mutable
        }
    }

    pub fn get_value(&self) -> String {
        match &self.definition {
            Definition::Range { value, .. } => value.to_string(),
            Definition::Selection {
                options, selected, ..
            } => options[*selected].clone(),
            Definition::Toggle(b) => b.to_string(),
            Definition::FixedNumber(num) => num.to_string(),
            Definition::FixedString(s) => s.to_string(),
        }
    }

    pub fn increment(&mut self) {
        if !self.is_mutable() {
            return;
        }
        match &mut self.definition {
            Definition::Range {
                min,
                max,
                step,
                value,
                ..
            } => {
                *value = (*value + *step).clamp(*min, *max);
            }
            Definition::Selection {
                options, selected, ..
            } => {
                *selected = if *selected == 0 {
                    options.len() - 1
                } else {
                    *selected - 1
                }
            }
            Definition::Toggle(b) => *b = !*b,
            _ => unreachable!("Tried to modify a non-mutable definition"),
        }
    }

    pub fn decrement(&mut self) {
        if !self.is_mutable() {
            return;
        }
        match &mut self.definition {
            Definition::Range {
                min,
                max,
                step,
                value,
                ..
            } => {
                *value = (*value - *step).clamp(*min, *max);
            }
            Definition::Selection {
                options, selected, ..
            } => *selected = (*selected + 1) % options.len(),
            Definition::Toggle(b) => *b = !*b,
            _ => unreachable!("Tried to modify a non-mutable definition"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Definition {
    Range {
        #[serde(default)]
        min: i64,
        #[serde(default = "default_range_max")]
        max: i64,
        #[serde(default = "default_range_step")]
        step: i64,
        default: Option<i64>,
        #[serde(skip)]
        value: i64,
    },
    Selection {
        options: Vec<String>,
        default: Option<String>,
        #[serde(skip)]
        selected: usize,
    },
    Toggle(bool),
    FixedNumber(i64),
    FixedString(String),
}

impl Definition {
    const fn is_mutable(&self) -> bool {
        !matches!(self, Self::FixedNumber(_) | Self::FixedString(_))
    }

    pub fn into_parameter(mut self, mutable: bool) -> Result<Parameter, ParameterError> {
        self.set_default_value()?;
        Ok(Parameter {
            definition: self,
            mutable,
        })
    }

    fn set_default_value(&mut self) -> Result<(), ParameterError> {
        self.evaluate().map(|_| match self {
            Self::Range {
                min,
                default,
                value,
                ..
            } => {
                if let Some(d) = default {
                    *value = *d;
                } else {
                    *value = *min;
                }
            }
            Self::Selection {
                options,
                default,
                selected,
            } => {
                if let Some(d) = default
                    && let Some(select) = options.iter().position(|opt| opt == d)
                {
                    *selected = select;
                } else {
                    *selected = 0;
                }
            }
            _ => (),
        })
    }

    fn evaluate(&self) -> Result<(), ParameterError> {
        match self {
            Self::Range {
                min,
                max,
                step,
                default,
                ..
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
            Self::Selection {
                options, default, ..
            } => {
                if options.is_empty() {
                    return Err(ParameterError::EmptySelection);
                }

                if default.as_ref().is_some_and(|d| !options.contains(&d)) {
                    return Err(ParameterError::DefaultNonExistant);
                }
            }
            _ => (),
        }

        Ok(())
    }
}

pub const fn default_range_step() -> i64 {
    1
}

pub const fn default_range_max() -> i64 {
    i64::MAX
}

#[cfg(test)]
mod test {
    use crate::config::parameters::Definition;

    #[test]
    fn increment_range() {
        let range = Definition::Range {
            min: 0,
            max: 10,
            step: 1,
            default: 5,
            value: 0,
        }
        .into_parameter(true);
    }
}
