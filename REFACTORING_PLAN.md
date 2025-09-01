# OctoType Refactoring & Feature Completion Plan

## Overview

This document outlines the refactoring plan to make OctoType feature-complete
and more maintainable before release. The focus is on config-driven
functionality, better code organization, and implementing missing features.

## 🚨 CURRENT STATUS (2025-08-29)

**Codebase State**: ✅ **WORKING** - Compiles and runs successfully

**What's Working**: ✅
- Page trait → enum conversion (completed successfully)
- Statistics configuration structure added to Config
- App structure updated to work with Page enum
- **FIXED**: Complete external source system implemented
- **FIXED**: Full mode system with built-in defaults (normal, quick_practice, timed_challenge)
- **FIXED**: Session factory for unified session creation
- **FIXED**: All import errors resolved (TextTheme, Source, SourceError)
- **FIXED**: Backward-compatible menu system

**What's Complete**: ✅
- External source system with command execution, parsing, and error handling
- Built-in default sources (quotes, random_words) with fallback handling
- Mode configuration system with parameters and win-conditions
- Session integration with mode support and completion logic
- All compilation errors resolved

**Next Steps** (for future TUI integration):
1. Connect new backend systems to TUI interfaces
2. Implement statistics persistence to JSON files
3. Add mode customization UI

**Progress on Refactoring Plan**: ~75% complete (Backend systems ✅, TUI integration pending)

## Missing Features (from README)

- [x] Multiple modes (Normal, Timed, and custom) - **IMPLEMENTED**: Built-in default modes with extensible config system
- [ ] Ability to save statistics to track improvements - **BACKEND READY**: Statistics framework in place, TUI integration pending
- [ ] Nix flake package
- [ ] Nix flake home-manager module

## Current Architecture Overview

```
src/
├── main.rs                    # CLI parsing, config loading, app initialization
├── config.rs                  # Theme and StatisticsConfig structs (✅ updated)
├── config/
│   └── theme.rs              # Theme configuration
├── sources.rs                 # Empty file - source system removed (❌ BROKEN)
├── utils.rs                   # Helper functions and traits
├── app.rs                     # App struct with Page enum (✅ updated)
└── page/                      # Page implementations
    ├── mod.rs                 # Page enum (✅ converted from trait)
    ├── error.rs               # Error display page
    ├── loadscreen.rs          # Loading screen with spinner
    ├── menu.rs                # Main menu (❌ BROKEN - references deleted sources)
    └── session/               # Typing session logic
        ├── mod.rs             # Main typing session
        ├── text.rs            # Text segment handling (❌ BROKEN - missing TextTheme)
        └── stats.rs           # Statistics calculation and display
```

**Current State**: Codebase is in transitional state - Page enum conversion succeeded but source system removal broke compilation.

## [✅] 1. Architecture Overhaul: Replace Page Trait with Enum

### ✅ COMPLETED - Page Enum Conversion

The Page trait system has been successfully converted to an enum-based approach as planned:

```rust
// Current trait-based approach
pub trait Page {
    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config);
    fn render_top(&mut self, config: &Config) -> Option<Line>;
    fn handle_events(&mut self, event: &Event, config: &Config) -> Option<Message>;
    fn poll(&mut self, config: &Config) -> Option<Message>;
    fn boxed(self) -> Box<Self> where Self: Sized;
}

pub struct App {
    page: Box<dyn Page>,  // Runtime overhead
    config: Rc<Config>,
}
```

### Problems with Current Approach

1. **Dynamic dispatch overhead**: `Box<dyn Page>` and vtable lookups
2. **Heap allocation**: Each page requires boxing
3. **Over-engineering**: No external consumers will implement this trait
4. **Limited compile-time checking**: Can't exhaustively match on page types
5. **Debugging complexity**: Indirect access makes tracing harder

### Proposed Solution: Enum-Based Pages

```rust
// New enum-based approach
#[derive(Debug)]
pub enum AppPage {
    Menu(Menu),
    LoadingScreen(LoadingScreen),
    TypingSession(TypingSession),
    Stats(Stats),
    Error(Error),
}

impl AppPage {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        match self {
            Self::Menu(page) => page.render(frame, area, config),
            Self::LoadingScreen(page) => page.render(frame, area, config),
            Self::TypingSession(page) => page.render(frame, area, config),
            Self::Stats(page) => page.render(frame, area, config),
            Self::Error(page) => page.render(frame, area, config),
        }
    }
    
    pub fn render_top(&mut self, config: &Config) -> Option<Line> {
        match self {
            Self::Menu(page) => page.render_top(config),
            Self::LoadingScreen(page) => page.render_top(config),
            Self::TypingSession(page) => page.render_top(config),
            Self::Stats(page) => page.render_top(config),
            Self::Error(page) => page.render_top(config),
        }
    }
    
    pub fn handle_events(&mut self, event: &Event, config: &Config) -> Option<Message> {
        match self {
            Self::Menu(page) => page.handle_events(event, config),
            Self::LoadingScreen(page) => page.handle_events(event, config),
            Self::TypingSession(page) => page.handle_events(event, config),
            Self::Stats(page) => page.render_top(config),
            Self::Error(page) => page.handle_events(event, config),
        }
    }
    
    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        match self {
            Self::Menu(page) => page.poll(config),
            Self::LoadingScreen(page) => page.poll(config),
            Self::TypingSession(page) => page.poll(config),
            Self::Stats(page) => page.poll(config),
            Self::Error(page) => page.poll(config),
        }
    }
}

// Simplified App struct
pub struct App {
    page: AppPage,  // No Box needed - zero-cost!
    config: Rc<Config>,
}

// Updated Message enum
pub enum Message {
    Error(Box<dyn std::error::Error + Send>),
    ShowPage(AppPage),  // Direct page instead of boxed trait
}
```

### Benefits of Enum Approach

1. **Zero-cost abstraction**: No runtime overhead, static dispatch
2. **Exhaustive matching**: Compiler ensures all page types are handled
3. **Better debugging**: Direct access to page data, clearer stack traces
4. **Type safety**: Can't accidentally miss handling a page type
5. **Performance**: No heap allocation or vtable lookups
6. **Simpler mental model**: All pages are known at compile time

### Migration Strategy

1. Replace `Page` trait with individual page implementations
2. Update `AppPage` enum with all page variants
3. Implement delegation methods on `AppPage`
4. Update `Message` enum to work with concrete types
5. Remove `boxed()` methods from all page implementations
6. Update all page transitions to use enum variants

✅ **COMPLETED**: This change has been successfully implemented and is working correctly.

## [🔄] 2. Configuration System Enhancements

### ✅ PARTIALLY COMPLETED - Statistics Config Added

```rust
// config.rs - Current structure (UPDATED)
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub theme: Theme,
    pub statistic: StatisticsConfig,  // ✅ ADDED
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatisticsConfig {
    pub save_enabled: bool,
    pub history_limit: usize,
    pub directory: Option<PathBuf>,
}
```

### Proposed Enhancement

```rust
// config.rs - Extended structure
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub theme: Theme,
    pub statistics: StatisticsConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StatisticsConfig {
    pub save_enabled: bool,
    pub history_limit: usize, // Number of sessions to keep
}
```

### Configuration File Example

```toml
[statistics]
save_enabled = true
history_limit = 1000

# Existing theme configuration remains unchanged
[theme]
spinner_color = "Yellow"
# ... rest of theme config
```

## 3. Advanced Mode System with TUI Customization

### ✅ IMPROVED ARCHITECTURE: Conditions vs Parameters

The mode system has been redesigned to clearly separate two distinct concepts:

- **🎯 Conditions** (`[mode.conditions]`): Define WHEN a session ends (win-conditions)
  - Time limits, word count goals, accuracy thresholds
  - Used by `is_complete()` method to determine session completion
  - Range and float range types for numeric goals

- **⚙️ Parameters** (`[mode.parameters]`): Define HOW a session behaves (customization)  
  - Text processing options, difficulty levels, categories
  - Used by source overrides to customize content generation
  - Select, toggle, text, and multi-select types for options

This separation eliminates confusion and creates a cleaner, more maintainable architecture.

### Current Session Structure

```rust
// session/mod.rs - Current structure
pub struct TypingSession {
    text: Vec<Segment>,
    current_segment_idx: usize,
    first_keypress: Option<Instant>,
    stats: RunningStats,
    // ... caches and state
}
```

### Enhanced Session with Mode Support

```rust
// session/mod.rs - Enhanced structure  
pub struct TypingSession {
    text: Vec<Segment>,
    current_segment_idx: usize,
    first_keypress: Option<Instant>,
    stats: RunningStats,
    mode: ResolvedModeConfig, // Resolved from config + user parameters
    // ... existing caches
}

#[derive(Debug, Clone)]
pub struct ResolvedModeConfig {
    pub name: String,
    pub source_overrides: HashMap<String, SourceOverride>,
    pub pipeline: Vec<PipelineStep>,
    pub parameter_values: ParameterValues,
    pub conditions: ConditionValues,  // Separate win-conditions
}

impl ResolvedModeConfig {
    pub fn is_complete(&self, session: &TypingSession) -> bool {
        // Check win-conditions (NOT parameters)
        if let Some(time_limit) = self.conditions.get_duration("time_limit") {
            if let Some(start) = session.first_keypress {
                return start.elapsed() >= time_limit;
            }
        }
        
        if let Some(word_count) = self.conditions.get_integer("word_count") {
            let typed_words = session.stats.word_count();
            if typed_words >= word_count as usize {
                return true;
            }
        }
        
        if let Some(accuracy_threshold) = self.conditions.get_float("accuracy_threshold") {
            let current_accuracy = session.stats.accuracy();
            let min_chars = self.conditions.get_integer("min_chars").unwrap_or(50);
            if session.stats.char_count() >= min_chars as usize {
                return current_accuracy >= accuracy_threshold;
            }
        }
        
        // Default: all segments completed (infinite mode)
        session.text.iter().all(|seg| seg.is_done())
    }
}
```

### Advanced Mode Configuration with TUI Customization

The mode system supports both static configuration and dynamic user
customization through the TUI:

#### Configuration Structure

```toml
# ~/.config/octotype/modes/timed_challenge.toml
[mode]
name = "Timed Challenge"
description = "Race against the clock with customizable duration"
source = "quotes"

# Static configuration (not adjustable in TUI)
pipeline = [
    { builtin = "shuffle" },
    { builtin = "ensure_min_length", args = ["4"] }
]

# Win-conditions: when the session ends
[mode.conditions]
time_limit = { min = 30, max = 900, default = 300, step = 30, suffix = "seconds" }

# Customization parameters: how the session behaves
[mode.parameters]
text_processing = { 
    options = ["normal", "no_punctuation", "lowercase"], 
    default = "normal" 
}
difficulty = {
    options = ["easy", "medium", "hard"],
    default = "medium"
}

# Template usage with parameters (NOT conditions)
[mode.source_overrides.quotes]
args = [
    "--processing", "{text_processing}",
    "--difficulty", "{difficulty}"
]
```

#### Parameter Type System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModeConfig {
    pub name: String,
    pub description: Option<String>,
    pub parameters: HashMap<String, ParameterDefinition>,     // Customization options
    pub conditions: HashMap<String, ConditionDefinition>,     // Win-conditions
    pub source_overrides: HashMap<String, SourceOverride>,
    pub pipeline: Vec<PipelineStep>,
    pub source: Option<String>,
}

// Parameter definitions for customization options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ParameterDefinition {
    Select {
        options: Vec<String>,
        default: String,
    },
    MultiSelect {
        options: Vec<String>,
        default: Vec<String>,
        min_selections: Option<usize>,
        max_selections: Option<usize>,
    },
    Toggle {
        default: bool,
        label: String,
    },
    Text {
        default: String,
        max_length: Option<usize>,
        pattern: Option<String>, // Regex validation
    },
}

// Condition definitions for win-conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionDefinition {
    Range {
        min: i32,
        max: Option<i32>,  // None = unbounded range
        default: i32,
        step: Option<i32>,
        suffix: Option<String>, // "seconds", "words", "% accuracy", etc.
    },
    FloatRange {
        min: f64,
        max: f64,
        default: f64,
        step: f64,
        suffix: Option<String>, // "% accuracy", etc.
    },
}

// Runtime parameter values with type-safe access (customization)
#[derive(Debug, Clone)]
pub struct ParameterValues {
    values: HashMap<String, ParameterValue>,
}

impl ParameterValues {
    pub fn get_string(&self, key: &str) -> Option<&str> { /* ... */ }
    pub fn get_string_list(&self, key: &str) -> Option<&[String]> { /* ... */ }
    pub fn get_boolean(&self, key: &str) -> Option<bool> { /* ... */ }
}

// Runtime condition values with type-safe access (win-conditions)
#[derive(Debug, Clone)]
pub struct ConditionValues {
    values: HashMap<String, ConditionValue>,
}

impl ConditionValues {
    pub fn get_integer(&self, key: &str) -> Option<i32> { /* ... */ }
    pub fn get_float(&self, key: &str) -> Option<f64> { /* ... */ }
    pub fn get_duration(&self, key: &str) -> Option<Duration> { /* ... */ }
}
```

#### TUI Integration Examples

```rust
// Enhanced menu handles mode customization
#[derive(Debug)]
pub enum MenuState {
    ModeSelection { selected: usize },
    ModeCustomization { 
        mode: ModeConfig, 
        parameter_values: ParameterValues,
        condition_values: ConditionValues,
        selected_section: CustomizationSection, // Parameters vs Conditions
        selected_item: usize 
    },
    SourceSelection { selected: usize },
    Ready { 
        resolved_mode: ResolvedModeConfig,
        source: Source 
    },
}

#[derive(Debug)]
pub enum CustomizationSection {
    Conditions,  // Win-conditions (when session ends)
    Parameters,  // Customization options (how session behaves)
}

// User sees separated customization interface:
// ┌─ Customize: Timed Challenge ────────────────┐
// │ 📍 WIN CONDITIONS (when session ends):      │
// │ > time_limit: 300 seconds    [←→ to adjust] │
// │                                             │
// │ ⚙️  PARAMETERS (session behavior):           │
// │   text_processing: normal    [←→ to cycle]  │
// │   difficulty: medium         [←→ to cycle]  │
// │   include_symbols: false     [Space toggle] │
// │                                             │
// │ [Enter] to confirm, [Esc] to go back       │
// └─────────────────────────────────────────────┘
```

#### Win-Condition Examples

```toml
# Time-based challenge
[mode.conditions]
time_limit = { min = 30, max = 1800, default = 300, step = 30, suffix = "seconds" }

# Word count goal
[mode.conditions]
word_count = { min = 10, max = 500, default = 100, step = 10, suffix = "words" }

# Accuracy challenge
[mode.conditions]
accuracy_threshold = { min = 80.0, max = 100.0, default = 95.0, step = 1.0, suffix = "% accuracy" }
min_chars = { min = 50, max = 1000, default = 100, step = 50, suffix = "characters" }

# Marathon mode (unbounded)
[mode.conditions]
marathon_duration = { min = 300, step = 60, default = 1800, suffix = "seconds" }
```

#### Parameter Examples

```toml
# Text processing options
[mode.parameters]
text_processing = { 
    options = ["normal", "no_punctuation", "lowercase"], 
    default = "normal" 
}

# Difficulty selection
difficulty = {
    options = ["beginner", "intermediate", "advanced"],
    default = "intermediate"
}

# Multi-select categories
categories = { 
    type = "multi_select",
    options = ["quotes", "code", "literature", "technical"],
    default = ["quotes"],
    min_selections = 1,
    max_selections = 3
}

# Custom theme name
custom_theme = { 
    type = "text", 
    default = "default", 
    max_length = 20,
    pattern = "^[a-zA-Z_][a-zA-Z0-9_]*$"
}
```

### Example Mode Configurations

#### 1. Time Challenge Mode
```toml
[mode]
name = "Time Challenge"
description = "Type as much as possible within a time limit"
source = "quotes"

[mode.conditions]
time_limit = { min = 30, max = 1800, default = 300, step = 30, suffix = "seconds" }

[mode.parameters]
text_processing = { options = ["normal", "no_punctuation"], default = "normal" }
difficulty = { options = ["easy", "medium", "hard"], default = "medium" }
```

#### 2. Accuracy Challenge Mode  
```toml
[mode]
name = "Accuracy Challenge"
description = "Maintain high accuracy over a minimum text length"
source = "random_words"

[mode.conditions]
accuracy_threshold = { min = 85.0, max = 99.0, default = 95.0, step = 1.0, suffix = "% accuracy" }
min_chars = { min = 100, max = 1000, default = 200, step = 50, suffix = "characters" }

[mode.parameters]
word_length = { options = ["short", "medium", "long"], default = "medium" }
include_numbers = { type = "toggle", default = false, label = "Include numbers" }
```

#### 3. Sprint Mode
```toml
[mode]
name = "Sprint Mode"
description = "Type a fixed number of words as fast as possible"
source = "processed_quotes"

[mode.conditions]
word_count = { min = 25, max = 200, default = 50, step = 25, suffix = "words" }

[mode.parameters]
text_processing = { options = ["normal", "lowercase"], default = "normal" }
categories = { 
    type = "multi_select",
    options = ["motivational", "technical", "literature"],
    default = ["motivational"],
    max_selections = 2
}
```

#### 4. Infinite Practice Mode
```toml
[mode]
name = "Practice Mode"
description = "Continuous typing practice with no end condition"
source = "system_words"

# No conditions = infinite mode (only ends when user quits)
[mode.conditions]

[mode.parameters]
word_difficulty = { options = ["common", "uncommon", "advanced"], default = "common" }
shuffle_order = { type = "toggle", default = true, label = "Shuffle word order" }
```

### Mode-Source Integration Benefits

1. **Clear Separation**: Conditions determine WHEN to stop, parameters determine HOW to behave
2. **Type Safety**: Parameter and condition definitions enforce valid ranges and options
3. **Template Integration**: Parameters (NOT conditions) integrate with source overrides
4. **Simple Logic**: `is_complete()` only needs to check conditions, not parameters
5. **User Friendly**: TUI can separate "Set Goal" vs "Customize Session"
6. **Extensible**: Easy to add new condition types (WPM threshold, error limit, etc.)

## 4. Statistics Persistence System

### Storage Approach Decision

After evaluating multiple storage options (SQLite, multiple files, binary
formats), **single JSON file** is the recommended approach for OctoType because:

1. **Right scale**: Even power users unlikely to exceed 10,000 sessions (~2MB)
2. **User-friendly**: Human-readable, easy to backup/sync
3. **Zero dependencies**: No additional crates required
4. **Debuggable**: Users can inspect/modify their data manually
5. **Simple**: Fits terminal app philosophy

### Current Statistics Flow

```
TypingSession → RunningStats → Stats (display only)
```

### Enhanced Statistics Flow

```
TypingSession → RunningStats → Stats → StatisticsManager → JSON File Storage
                                   ↓
                              Historical Analysis
```

### JSON File Structure

```json
{
  "version": 1,
  "sessions": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "mode": "timed",
      "duration_minutes": 1.0,
      "wpm_raw": 85.2,
      "wpm_actual": 82.1,
      "accuracy": 96.5,
      "word_count": 85,
      "error_count": 3
    }
  ],
  "personal_bests": {
    "highest_wpm": 95.2,
    "highest_accuracy": 98.1,
    "longest_session": 15.5
  },
  "metadata": {
    "total_sessions": 1,
    "total_time_minutes": 1.0,
    "created": "2024-01-15T10:00:00Z",
    "last_updated": "2024-01-15T10:30:00Z"
  }
}
```

### Implementation

```rust
// New file: src/statistics.rs
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub mode: TypingMode,
    pub duration_minutes: f64,
    pub wpm_raw: f64,
    pub wpm_actual: f64,
    pub accuracy: f64,
    pub word_count: usize,
    pub error_count: u16,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersonalBests {
    pub highest_wpm: f64,
    pub highest_accuracy: f64,
    pub longest_session: f64,
    pub total_sessions: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsFile {
    pub version: u32,
    pub sessions: VecDeque<SessionRecord>,
    pub personal_bests: PersonalBests,
    pub metadata: Metadata,
}

pub struct StatisticsManager {
    data: StatisticsFile,
    file_path: PathBuf,
    max_sessions: usize, // Configurable limit (default: 10,000)
    dirty: bool, // Only write when data changes
}

impl StatisticsManager {
    /// Load existing statistics file or create a new one
    pub fn load_or_create(config_dir: &Path, max_sessions: usize) -> Result<Self, StatisticsError>;
    
    /// Add a new session record with automatic cleanup
    pub fn add_session(&mut self, session: SessionRecord);
    
    /// Save changes to disk using atomic write
    pub fn save(&mut self) -> Result<(), StatisticsError>;
    
    /// Get recent performance trends
    pub fn get_trends(&self, last_n_sessions: usize) -> TrendAnalysis;
    
    /// Get statistics for a specific time period
    pub fn get_period_stats(&self, since: chrono::DateTime<chrono::Utc>) -> PeriodStats;
}
```

### Key Features of This Approach

1. **Atomic Writes**: Prevents data corruption through temp file + rename
2. **Automatic Cleanup**: VecDeque with configurable size limit prevents
   unbounded growth
3. **Backup Strategy**: Creates `.bak` file before writes
4. **Version Migration**: Built-in support for future schema changes
5. **Lazy Loading**: Only loads file when statistics are actually accessed
6. **Dirty Flag**: Only writes when data has changed
7. **Error Recovery**: Comprehensive error handling with specific error types

### File Location

```
~/.config/octotype/
├── config.toml
├── statistics.json      # Main statistics file
├── statistics.json.bak  # Automatic backup
└── statistics.json.tmp  # Temporary file during writes (removed after)
```

## 5. External-Only Word Source System

### Current Limitation

The current source system is hardcoded and inflexible with built-in network
dependencies.

Users cannot add their own word sources without modifying the binary.

### Plugin System Decision

After evaluating multiple approaches (embedded scripting with Rhai/Lua, WASM
plugins, JSON templates, dynamic libraries), **external process interface** is
recommended because:

1. **Zero binary bloat**: No embedded runtime dependencies
2. **Maximum flexibility**: Users can write in any language
3. **Familiar tooling**: Standard development environment
4. **Right complexity**: Suitable for technical terminal users
5. **Unix philosophy**: Leverage existing tools and compose functionality

### Enhanced Source Architecture

```rust
// New external-only source system
#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalSource {
    pub name: String,
    pub command: Vec<String>,
    pub timeout_seconds: u64,
    pub output_format: OutputFormat,
    pub args_template: HashMap<String, String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OutputFormat {
    JsonArray,        // ["word1", "word2", "word3"]
    Lines,            // "word1\nword2\nword3\n"
    SpaceSeparated,   // "word1 word2 word3"
}

impl ExternalSource {
    pub fn fetch(&self, args: &SourceArgs) -> Result<Vec<String>, SourceError> {
        let mut cmd = std::process::Command::new(&self.command[0]);
        cmd.args(&self.command[1..]);
        
        // Template substitution for dynamic arguments
        for (key, template) in &self.args_template {
            let value = self.substitute_template(template, args);
            cmd.arg(format!("--{}", key)).arg(value);
        }
        
        // Execute with timeout
        let output = cmd
            .timeout(Duration::from_secs(self.timeout_seconds))
            .output()
            .map_err(|e| SourceError::ExternalCommand { 
                command: self.command[0].clone(), 
                error: e 
            })?;
            
        if !output.status.success() {
            return Err(SourceError::ExternalCommandFailed {
                command: self.command[0].clone(),
                exit_code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }
        
        self.parse_output(&output.stdout)
    }
    
    fn substitute_template(&self, template: &str, args: &SourceArgs) -> String {
        // Replace {word_count}, {difficulty}, etc. with actual values
        template
            .replace("{word_count}", &args.word_count.unwrap_or(50).to_string())
            .replace("{max_length}", &args.max_length.unwrap_or(10).to_string())
    }
    
    fn parse_output(&self, output: &[u8]) -> Result<Vec<String>, SourceError> {
        let content = String::from_utf8_lossy(output);
        
        match self.output_format {
            OutputFormat::JsonArray => {
                serde_json::from_str(&content).map_err(SourceError::JsonParse)
            }
            OutputFormat::Lines => {
                Ok(content.lines().map(|s| s.trim().to_string()).collect())
            }
            OutputFormat::SpaceSeparated => {
                Ok(content.split_whitespace().map(|s| s.to_string()).collect())
            }
        }
    }
}
```

### User Configuration Examples

#### Simple File-Based Source

```toml
# ~/.config/octotype/sources/my_words.toml
[source]
name = "My Custom Word List"
command = ["cat", "/home/user/typing_practice/words.txt"]
timeout_seconds = 5
output_format = "lines"
description = "Personal collection of difficult words"

# No dynamic args needed - just reads file
```

#### Dynamic Script-Based Source

```toml
# ~/.config/octotype/sources/programming.toml
[source]
name = "Programming Terms"
command = ["python3", "/home/user/.config/octotype/scripts/programming_words.py"]
timeout_seconds = 30
output_format = "json"
description = "Programming vocabulary by language and difficulty"

[source.args]
count = "{word_count}"
difficulty = "{difficulty}"
language = "rust"
```

#### Corresponding User Script

```python
#!/usr/bin/env python3
# ~/.config/octotype/scripts/programming_words.py
import sys
import json
import random
import argparse

def get_programming_words(count, difficulty, language):
    """Generate programming-related words based on parameters."""
    
    word_sets = {
        "rust": {
            "easy": ["fn", "let", "mut", "pub", "use", "mod", "impl"],
            "medium": ["trait", "lifetime", "borrowing", "ownership", "closure"],
            "hard": ["associated_type", "higher_ranked", "phantom_data"]
        },
        "python": {
            "easy": ["def", "class", "import", "return", "if", "for", "while"],
            "medium": ["decorator", "generator", "comprehension", "lambda"],
            "hard": ["metaclass", "descriptor", "async", "await"]
        }
    }
    
    words = word_sets.get(language, {}).get(difficulty, [])
    return random.sample(words, min(count, len(words)))

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--count", type=int, default=50)
    parser.add_argument("--difficulty", default="easy")
    parser.add_argument("--language", default="rust")
    
    args = parser.parse_args()
    
    try:
        words = get_programming_words(args.count, args.difficulty, args.language)
        print(json.dumps(words))
    except Exception as e:
        sys.stderr.write(f"Error: {e}\n")
        sys.exit(1)
```

#### Web API Integration

```toml
# ~/.config/octotype/sources/news_headlines.toml
[source]
name = "News Headlines"
command = ["curl", "-s", "https://api.example.com/headlines"]
timeout_seconds = 10
output_format = "json"
description = "Current news headlines for typing practice"

# Could also use a wrapper script for more complex API handling
```

### Source Discovery & Management

```rust
// External source loading system
pub struct SourceManager {
    sources: Vec<ExternalSource>,
    config_dir: PathBuf,
}

impl SourceManager {
    pub fn load_all_sources(config_dir: &Path) -> Result<Self, SourceError> {
        let sources = Self::discover_external_sources(config_dir)?;
        
        Ok(Self {
            sources,
            config_dir: config_dir.to_path_buf(),
        })
    }
    
    fn discover_external_sources(config_dir: &Path) -> Result<Vec<ExternalSource>, SourceError> {
        let sources_dir = config_dir.join("sources");
        if !sources_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut sources = Vec::new();
        
        for entry in std::fs::read_dir(&sources_dir)? {
            let path = entry?.path();
            if path.extension().map_or(false, |ext| ext == "toml") {
                let content = std::fs::read_to_string(&path)?;
                let config: ExternalSourceConfig = toml::from_str(&content)?;
                sources.push(config.source);
            }
        }
        
        Ok(sources)
    }
    
    pub fn get_all_sources(&self) -> impl Iterator<Item = &ExternalSource> {
        self.sources.iter()
    }
    
    pub fn get_source_by_name(&self, name: &str) -> Option<&ExternalSource> {
        self.sources.iter().find(|s| s.name == name)
    }
}

#[derive(Deserialize)]
struct ExternalSourceConfig {
    source: ExternalSource,
}
```

### Enhanced Error Handling for External Sources

```rust
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    // Existing errors...
    #[error("External command '{command}' failed to execute: {error}")]
    ExternalCommand { command: String, error: std::io::Error },
    
    #[error("External command '{command}' failed with exit code {exit_code:?}: {stderr}")]
    ExternalCommandFailed { 
        command: String, 
        exit_code: Option<i32>, 
        stderr: String 
    },
    
    #[error("Command timeout after {timeout_seconds} seconds")]
    Timeout { timeout_seconds: u64 },
    
    #[error("Failed to parse command output as JSON: {0}")]
    JsonParse(serde_json::Error),
    
    #[error("No words returned from source")]
    EmptyOutput,
}
```

### Directory Structure

```
~/.config/octotype/
├── config.toml              # Main configuration
├── statistics.json          # Statistics storage
├── modes/                   # Mode definitions (ships with defaults)
│   ├── normal.toml          # Default mode using quotes
│   ├── quick_practice.toml  # Random words practice
│   └── timed_challenge.toml # Timed mode with quotes
└── sources/                 # External source definitions (ships with defaults)
    ├── quotes.toml          # Replaces built-in Quote API
    ├── random_words.toml    # Replaces built-in RandomWords API
    ├── processed_quotes.toml # Enhanced quotes with processing options
    └── system_words.toml    # Offline fallback using system dictionary
```

### Benefits of This Approach

1. **Zero Dependencies**: No embedded scripting runtime
2. **Ultimate Flexibility**: Users can integrate any data source
3. **Familiar Development**: Use existing tools and languages
4. **Secure by Design**: Process isolation with timeouts
5. **Easy Distribution**: Users can share `.toml` configs
6. **Gradual Adoption**: Built-in sources work without setup
7. **Unix Philosophy**: Compose small tools effectively

### Replacing Built-in Sources with External Configurations

Instead of maintaining hardcoded network sources, the current Quote and
RandomWords APIs will be reimplemented as external source configurations using
standard Unix tools.

#### Quote API as External Source

```toml
# ~/.config/octotype/sources/quotes.toml (shipped as default)
[source]
name = "Inspirational Quotes"
command = ["sh", "-c", "curl -s 'https://api.quotable.kurokeita.dev/api/quotes/random' | jq -r '.quote.content' | tr ' ' '\\n'"]
timeout_seconds = 10
output_format = "lines"
description = "Famous quotes and inspirational sayings"

# No dynamic arguments needed - API returns random quotes
```

#### Random Words API as External Source

```toml
# ~/.config/octotype/sources/random_words.toml (shipped as default)
[source]
name = "Random English Words"
command = ["bash", "-c", """
    count=${COUNT:-50}
    max_length=${MAX_LENGTH:-15}
    words=()
    
    while [ ${#words[@]} -lt $count ]; do
        word=$(curl -s 'https://random-word-api.herokuapp.com/word' | jq -r '.[0]')
        if [ -z '$max_length' ] || [ ${#word} -le $max_length ]; then
            words+=("$word")
        fi
    done
    
    printf '%s\\n' "${words[@]}"
"""]
timeout_seconds = 60
output_format = "lines"
description = "Random words filtered by length"

[source.args]
count = "{word_count}"
max_length = "{max_word_length}"
```

#### Enhanced Quote Processing

```toml
# ~/.config/octotype/sources/processed_quotes.toml
[source]
name = "Processed Quotes"
command = ["bash", "-c", """
    # Fetch quote with fallback
    quote=$(curl -s --max-time 5 'https://api.quotable.kurokeita.dev/api/quotes/random' | jq -r '.quote.content' 2>/dev/null)
    
    # Fallback to secondary API
    if [ -z \"$quote\" ] || [ \"$quote\" = \"null\" ]; then
        quote=$(curl -s --max-time 5 'https://zenquotes.io/api/random' | jq -r '.[0].q' 2>/dev/null)
    fi
    
    # Final fallback
    if [ -z \"$quote\" ] || [ \"$quote\" = \"null\" ]; then
        quote=\"The quick brown fox jumps over the lazy dog. Pack my box with five dozen liquor jugs.\"
    fi
    
    # Process based on parameters
    case '${PROCESSING:-normal}' in
        'no_punctuation')
            echo \"$quote\" | tr -d '[:punct:]' | tr ' ' '\\n'
            ;;
        'lowercase')
            echo \"$quote\" | tr '[:upper:]' '[:lower:]' | tr ' ' '\\n'
            ;;
        *)
            echo \"$quote\" | tr ' ' '\\n'
            ;;
    esac
"""]
timeout_seconds = 15
output_format = "lines"
description = "Quotes with multiple fallback sources and processing options"

[source.args]
processing = "{text_processing}"
```

#### Offline/Local Sources for Reliability

```toml
# ~/.config/octotype/sources/system_words.toml
[source]
name = "System Dictionary"
command = ["bash", "-c", """
    dict_file="/usr/share/dict/words"
    if [ ! -f "$dict_file" ]; then
        dict_file="/usr/dict/words"
    fi
    
    if [ -f "$dict_file" ]; then
        shuf "$dict_file" | head -n ${COUNT:-50}
    else
        # Fallback word list
        echo -e "the\\nquick\\nbrown\\nfox\\njumps\\nover\\nlazy\\ndog"
    fi
"""]
timeout_seconds = 5
output_format = "lines"
description = "Words from system dictionary (offline)"

[source.args]
count = "{word_count}"
```

### Default Modes Using External Sources

```toml
# ~/.config/octotype/modes/normal.toml (ships with binary)
[mode]
name = "Normal"
description = "Classic typing practice with quotes"
source = "quotes"

[mode.parameters]
text_processing = { 
    options = ["normal", "no_punctuation", "lowercase"], 
    default = "normal" 
}

[mode.source_overrides.processed_quotes]
args = ["--processing", "{text_processing}"]
```

```toml
# ~/.config/octotype/modes/quick_practice.toml (ships with binary)
[mode] 
name = "Quick Practice"
description = "Short practice with random words"
source = "random_words"

[mode.parameters]
word_count = { min = 10, max = 100, default = 25, step = 5, suffix = "words" }
max_word_length = { min = 3, max = 15, default = 8, step = 1, suffix = "letters" }

[mode.source_overrides.random_words]
args = [
    "--count", "{word_count}",
    "--max-length", "{max_word_length}"
]
```

### Benefits of External-Only Approach

1. **Smaller Binary**: No HTTP client, JSON parsing, or API-specific code
2. **Reliability**: Users can modify sources to add fallbacks and error handling
3. **Debuggability**: Sources can be tested independently with standard tools
4. **Flexibility**: Easy to switch APIs, add preprocessing, combine sources
5. **Unix Philosophy**: Compose small, focused tools
6. **Offline Capable**: Can easily add local/offline sources

### Migration Strategy

1. **Remove hardcoded sources**: Delete existing Quote and RandomWords
   implementations
2. **Ship default external configs**: Include the TOML files above in the binary
   installation
3. **Fallback handling**: Ensure graceful degradation when external tools are
   missing
4. **Documentation**: Provide clear examples of how to customize sources
5. **Validation**: Check for required tools (curl, jq) on first run and suggest
   alternatives

### Source Validation and Fallbacks

```rust
// Check for required external tools
pub fn validate_source_requirements(source: &ExternalSource) -> Vec<String> {
    let mut missing = Vec::new();
    
    // Check if command exists
    if let Some(cmd) = source.command.first() {
        if which::which(cmd).is_err() {
            missing.push(format!("Command '{}' not found", cmd));
        }
    }
    
    // Suggest alternatives for common missing tools
    if missing.iter().any(|m| m.contains("jq")) {
        missing.push("Install jq with: apt install jq (Ubuntu) or brew install jq (macOS)".to_string());
    }
    
    missing
}
```

This approach completely eliminates hardcoded network dependencies while
providing users with working configurations that demonstrate the system's
capabilities.

## 6. Unified Error Handling Protocol

To ensure consistent error handling across all external sources, a standardized
communication protocol is established:

#### Error Output Format for External Scripts

External sources should communicate errors via stderr using structured key-value
pairs:

```bash
#!/bin/bash
# Example: Well-behaved external source error handling

# Success case - output words to stdout
if successful_operation; then
    echo "word1"
    echo "word2"
    exit 0
fi

# Error cases - structured output to stderr, non-zero exit code
if network_failed; then
    echo "ERROR_TYPE=network" >&2
    echo "ERROR_MESSAGE=Could not connect to api.example.com" >&2
    echo "SUGGESTION=Check your internet connection or try 'system_words' source" >&2
    exit 1
fi

if ! command -v jq >/dev/null; then
    echo "ERROR_TYPE=missing_dependency" >&2
    echo "ERROR_MESSAGE=jq command not found" >&2
    echo "SUGGESTION=Install jq: apt install jq (Ubuntu) or brew install jq (macOS)" >&2
    exit 2
fi
```

#### Standard Error Types

- `network` - Connection/API failures
- `missing_dependency` - Required tools not installed
- `parse_error` - Invalid response format
- `config_error` - Configuration issues (missing files, etc.)
- `empty_result` - No data returned
- `timeout` - Operation exceeded time limit

#### Enhanced Source Configuration with Error Metadata

```toml
[source]
name = "Reliable Quotes"
command = ["bash", "-c", "..."]
# ... standard config ...

# Error handling metadata
[source.error_handling]
required_tools = ["curl", "jq", "bash"]
network_required = true
offline_alternative = "system_words"
typical_errors = ["network", "parse_error"]
max_retries = 2
retry_delay_seconds = 5
```

#### Minimal Rust Error Handling Implementation

```rust
#[derive(Debug, Clone)]
pub struct SourceExecutionResult {
    pub success: bool,
    pub words: Vec<String>,
    pub error_type: Option<String>,
    pub error_message: Option<String>,
    pub suggestion: Option<String>,
}

impl ExternalSource {
    pub fn execute(&self, args: &SourceArgs) -> SourceExecutionResult {
        let output = match std::process::Command::new(&self.command[0])
            .args(&self.command[1..])
            .output() {
                Ok(output) => output,
                Err(_) => return SourceExecutionResult::command_failed(&self.command[0])
            };

        if output.status.success() {
            SourceExecutionResult {
                success: true,
                words: self.parse_output(&output.stdout),
                error_type: None,
                error_message: None,
                suggestion: None,
            }
        } else {
            self.parse_error_output(&String::from_utf8_lossy(&output.stderr))
        }
    }

    fn parse_error_output(&self, stderr: &str) -> SourceExecutionResult {
        let mut error_type = None;
        let mut error_message = None;
        let mut suggestion = None;

        for line in stderr.lines() {
            if let Some(value) = line.strip_prefix("ERROR_TYPE=") {
                error_type = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("ERROR_MESSAGE=") {
                error_message = Some(value.to_string());
            } else if let Some(value) = line.strip_prefix("SUGGESTION=") {
                suggestion = Some(value.to_string());
            }
        }

        SourceExecutionResult {
            success: false,
            words: Vec::new(),
            error_type,
            error_message,
            suggestion,
        }
    }
}
```

#### Benefits of This Error Protocol

1. **Consistent UX**: All sources provide similarly formatted error information
2. **User-Friendly**: Clear messages with actionable suggestions
3. **Debuggable**: Developers can test error conditions independently
4. **Self-Documenting**: Error metadata helps users understand failure modes
5. **Minimal Complexity**: Simple key-value parsing in Rust
6. **Extensible**: Easy to add new error types as needed

## 7. Menu System Improvements

### Current Menu Issues

- Complex source selection UI
- Args system is type-unsafe
- Menu navigation could be clearer

### Simplified Menu Structure

```rust
// menu/mod.rs - Cleaner structure
pub struct Menu {
    state: MenuState,
    source_config: SourceConfig,
}

#[derive(Debug)]
pub enum MenuState {
    ModeSelection { selected: usize },
    SourceSelection { selected: usize },
    SourceConfiguration { source: ExternalSource, args: SourceArgs },
    Ready,
}

// Generic source arguments for external sources
#[derive(Debug, Clone)]
pub struct SourceArgs {
    pub word_count: Option<usize>,
    pub max_word_length: Option<usize>,
    pub difficulty: Option<String>,
    pub categories: Vec<String>,
    pub text_processing: Option<String>,
    // Extensible for any source-specific parameters
    pub custom_params: HashMap<String, String>,
}
```

## 8. Implementation Phases

### Phase 1: **COMPLETED** - Architectural Foundation

- [x] **Replace Page trait with AppPage enum** 
- [x] **Replace Source enum with ExternalSource struct**
- [x] **Remove all built-in source code completely** (Quote, RandomWords replaced with external sources)
- [x] **Implement external source system** with discovery and execution
- [ ] Fix all TODO comments and `expect()` calls
- [ ] Fix typos throughout codebase (accurracy → accuracy, BrialleDouble → BrailleDouble)
- [ ] Add proper unified error handling protocol

**RESOLVED**: All compilation issues have been fixed. The external source system is fully implemented and working.

### Phase 1.5: **COMPLETED** - Fixed Broken Compilation

- [x] **Implement minimal external source system** to replace deleted built-in sources
- [x] **Fix menu.rs compilation errors** (Source/SourceError references)
- [x] **Add missing TextTheme** to config system
- [x] **Create basic external source configurations** (quotes.toml, random_words.toml)
- [x] **Test that application compiles and runs**

### Phase 2: **COMPLETED** - Mode System Implementation

- [x] **Advanced mode configuration with TUI customization** - Backend ready, TUI integration pending
- [x] **Mode-source integration with external sources only** 
- [x] **Template system for dynamic parameters** (parameter substitution)
- [x] **ResolvedModeConfig implementation** for runtime mode resolution
- [x] Mode discovery system for loading TOML configurations

### Phase 3: Statistics and Error Handling

- [ ] **JSON-based statistics persistence** with StatisticsManager
- [ ] **Unified error handling protocol** for external sources
- [ ] **External source error communication** (stderr parsing with ERROR_TYPE
      format)
- [ ] Atomic writes and backup strategy for statistics
- [ ] Historical analysis and trends calculation

### Phase 4: Polish and Integration

- [ ] **Menu system improvements** (simplified with external-source awareness)
- [ ] **Default configurations shipping** (quotes.toml, random_words.toml,
      system_words.toml, default modes)
- [ ] **Source validation and fallback handling** (check for required tools like
      curl, jq)
- [ ] **Configuration system enhancements** (statistics config)
- [ ] Documentation and examples for custom sources and modes
- [ ] Create Nix flake and home-manager module

## Expected File Changes

### Files Already Modified ✅
- `src/page.rs` - Successfully converted to Page enum
- `src/app.rs` - Updated to work with Page enum
- `src/config.rs` - Added StatisticsConfig structure
- `src/sources.rs` - **EMPTIED** (causing compilation errors)
- Deleted: `src/sources/quote_api.rs`, `src/sources/random_words.rs`

### Current Broken State ❌
- `src/page/menu.rs` - References non-existent Source types
- `src/page/session/text.rs` - References missing TextTheme
- **Application does not compile**

### New Files

- `src/statistics.rs` - Statistics persistence and analysis
- `~/.config/octotype/sources/*.toml` - Default external source configurations
- `~/.config/octotype/modes/*.toml` - Default mode configurations

### Removed Files

- All built-in source implementations (Quote, RandomWords APIs)
- Network-related dependencies and code

### Modified Files

- `src/config.rs` - Extended configuration structure (statistics config)
- `src/app/session/mod.rs` - Mode system integration with ResolvedModeConfig
- `src/app/menu/mod.rs` - Simplified menu system for external sources
- `src/main.rs` - Statistics manager and source manager integration
- `Cargo.toml` - Remove: minreq, serde_json (network deps). Add: chrono,
  thiserror, which

## Benefits of This Plan

1. **Smaller Binary**: External-only approach eliminates HTTP client, JSON
   parsing, and network dependencies (~500KB+ reduction)
2. **Ultimate Flexibility**: Users can create any word source using any language
   or tool they prefer
3. **Unix Philosophy**: Compose small, focused tools rather than building
   monolithic functionality
4. **Offline Capability**: Easy to add local/offline sources (system dictionary,
   text files) with automatic fallbacks
5. **User Debuggable**: Sources can be tested independently with standard shell
   commands
6. **Zero Vendor Lock-in**: No dependency on specific APIs - users control their
   data sources completely
7. **Maintainability**: Better error handling, code organization, and eliminated
   network complexity
8. **Extensibility**: Config-driven modes and sources make adding features
   trivial
9. **User Experience**: Persistent statistics, multiple modes, and sophisticated
   customization through TUI
10. **Release Ready**: Addresses all missing features while simplifying the
    architecture

## Estimated Implementation Time

- Phase 1: ✅ Partially complete (Page enum ✅, Source system ❌)
- Phase 1.5: **URGENT** 1-2 days to fix compilation
- Phase 2: 2-3 days (Mode system implementation)
- Phase 3: 3-4 days (Statistics persistence)
- Phase 4: 2-3 days (Polish and default configurations)

**Remaining: ~8-12 days of development time**
**Next Priority: Fix broken compilation (Phase 1.5)**

