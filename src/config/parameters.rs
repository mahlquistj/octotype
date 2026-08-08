use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type ParameterDefinitions = HashMap<String, Definition>;

#[derive(Debug, Error, PartialEq, Eq)]
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

    #[error("Failed to parse value '{value}' for parameter '{name}': {reason}")]
    ParseValue {
        name: String,
        value: String,
        reason: String,
    },

    #[error("Value {value} out of bounds [{min}, {max}] for parameter '{name}'")]
    OutOfBounds {
        name: String,
        value: i64,
        min: i64,
        max: i64,
    },

    #[error("Invalid selection '{value}' for parameter '{name}'. Valid options: {valid}")]
    InvalidSelection {
        name: String,
        value: String,
        valid: String,
    },

    #[error("Parameter '{0}' is fixed and cannot be changed")]
    FixedParameter(String),

    #[error("Unknown parameter '{0}'")]
    UnknownParameter(String),
}

pub struct ParameterValues(HashMap<String, Parameter>);

impl ParameterValues {
    pub fn get(&self, key: &str) -> Option<&Parameter> {
        self.0.get(key)
    }

    pub fn replace_values(&self, string: &str) -> String {
        let mut result = String::new();
        let mut remaining = string;

        while let Some(start) = remaining.find('{') {
            result.push_str(&remaining[..start]);
            remaining = &remaining[start + 1..];

            if let Some(end) = remaining.find('}') {
                let key = &remaining[..end];
                if !key.is_empty() {
                    if let Some(param) = self.get(key) {
                        result.push_str(&param.get_value());
                    } else {
                        result.push('{');
                        result.push_str(key);
                        result.push('}');
                    }
                } else {
                    result.push_str("{}");
                }
                remaining = &remaining[end + 1..];
            } else {
                result.push('{');
                break;
            }
        }

        result.push_str(remaining);
        result
    }
}

impl FromIterator<(String, Parameter)> for ParameterValues {
    fn from_iter<T: IntoIterator<Item = (String, Parameter)>>(iter: T) -> Self {
        Self(HashMap::from_iter(iter))
    }
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

    pub fn set_value_from_str(
        &mut self,
        name: &str,
        value_str: &str,
    ) -> Result<(), ParameterError> {
        if !self.is_mutable() {
            return Err(ParameterError::FixedParameter(name.to_string()));
        }

        match &mut self.definition {
            Definition::Range { min, max, value, .. } => {
                let parsed: i64 = value_str.parse().map_err(|_| ParameterError::ParseValue {
                    name: name.to_string(),
                    value: value_str.to_string(),
                    reason: "expected an integer".to_string(),
                })?;

                if parsed < *min || parsed > *max {
                    return Err(ParameterError::OutOfBounds {
                        name: name.to_string(),
                        value: parsed,
                        min: *min,
                        max: *max,
                    });
                }

                *value = parsed;
            }
            Definition::Selection {
                options, selected, ..
            } => {
                let pos = options
                    .iter()
                    .position(|opt| opt == value_str || opt.eq_ignore_ascii_case(value_str))
                    .ok_or_else(|| ParameterError::InvalidSelection {
                        name: name.to_string(),
                        value: value_str.to_string(),
                        valid: options.join(", "),
                    })?;

                *selected = pos;
            }
            Definition::Toggle(b) => {
                let parsed: bool = match value_str.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "y" | "on" => true,
                    "false" | "0" | "no" | "n" | "off" => false,
                    _ => {
                        return Err(ParameterError::ParseValue {
                            name: name.to_string(),
                            value: value_str.to_string(),
                            reason: "expected a boolean (true/false)".to_string(),
                        });
                    }
                };

                *b = parsed;
            }
            Definition::FixedNumber(_) | Definition::FixedString(_) => {
                return Err(ParameterError::FixedParameter(name.to_string()));
            }
        }

        Ok(())
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

                if default.as_ref().is_some_and(|d| !options.contains(d)) {
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
mod tests {
    use super::*;

    #[test]
    fn test_set_range_value() {
        let def = Definition::Range {
            min: 10,
            max: 100,
            step: 5,
            default: Some(20),
            value: 20,
        };
        let mut param = def.into_parameter(true).expect("valid range parameter");
        assert_eq!(param.get_value(), "20");

        param
            .set_value_from_str("words", "50")
            .expect("valid value");
        assert_eq!(param.get_value(), "50");

        let err = param
            .set_value_from_str("words", "5")
            .expect_err("out of bounds");
        assert_eq!(
            err,
            ParameterError::OutOfBounds {
                name: "words".to_string(),
                value: 5,
                min: 10,
                max: 100
            }
        );

        let err = param
            .set_value_from_str("words", "abc")
            .expect_err("invalid int");
        assert_eq!(
            err,
            ParameterError::ParseValue {
                name: "words".to_string(),
                value: "abc".to_string(),
                reason: "expected an integer".to_string()
            }
        );
    }

    #[test]
    fn test_set_selection_value() {
        let def = Definition::Selection {
            options: vec!["easy".to_string(), "medium".to_string(), "hard".to_string()],
            default: Some("easy".to_string()),
            selected: 0,
        };
        let mut param = def
            .into_parameter(true)
            .expect("valid selection parameter");
        assert_eq!(param.get_value(), "easy");

        param
            .set_value_from_str("diff", "HARD")
            .expect("valid selection");
        assert_eq!(param.get_value(), "hard");

        let err = param
            .set_value_from_str("diff", "extreme")
            .expect_err("invalid selection");
        assert_eq!(
            err,
            ParameterError::InvalidSelection {
                name: "diff".to_string(),
                value: "extreme".to_string(),
                valid: "easy, medium, hard".to_string()
            }
        );
    }

    #[test]
    fn test_set_toggle_value() {
        let def = Definition::Toggle(false);
        let mut param = def.into_parameter(true).expect("valid toggle parameter");
        assert_eq!(param.get_value(), "false");

        param
            .set_value_from_str("toggle", "yes")
            .expect("valid toggle");
        assert_eq!(param.get_value(), "true");
    }

    #[test]
    fn test_set_fixed_value() {
        let def = Definition::FixedString("fixed".to_string());
        let mut param = def.into_parameter(false).expect("valid fixed parameter");

        let err = param
            .set_value_from_str("param", "val")
            .expect_err("fixed param");
        assert_eq!(err, ParameterError::FixedParameter("param".to_string()));
    }
}
