# Gladius - High-Performance Typing Trainer Library

Gladius is a comprehensive Rust library for building typing trainer
applications. It provides real-time typing analysis, flexible rendering systems,
and detailed performance statistics with a focus on accuracy, performance, and
ease of use.

**Gladius is the core library powering
[OctoType](https://github.com/mahlquistj/octotype)** and follows the same
versioning scheme. When OctoType releases version `0.3.2`, Gladius is also at
version `0.3.2`, ensuring compatibility and synchronized development. Gladius
might be split into it's own repository later.

## 🚀 Quick Start

```rust
use gladius::TypingSession;

// Create a typing session
let mut session = TypingSession::new("Hello, world!").unwrap();

// Process user input
while let Some((char, result)) = session.input(Some('H')) {
    println!("Typed '{}': {:?}", char, result);
    break; // Just for demo
}

// Get progress and statistics
println!("Progress: {:.1}%", session.completion_percentage());
println!("WPM: {:.1}", session.statistics().measurements.last()
    .map(|m| m.wpm.raw).unwrap_or(0.0));
```

## 💡 Key Features

### 🏃‍♂️ **High Performance**

- **Fast character processing** - Amortized O(1) keystroke handling
- **O(1) word lookups** - Efficient character-to-word mapping
- **Optimized statistics** - Welford's algorithm for numerical stability
- **Memory efficient** - Minimal allocations during typing

### 📊 **Comprehensive Statistics**

- **Words per minute** (raw, corrected, actual)
- **Input per minute** (raw, actual)
- **Accuracy percentages** (raw, actual)
- **Consistency analysis** with standard deviation
- **Detailed error tracking** by character and word
- **Real-time measurements** at configurable intervals

### 🎯 **Flexible Rendering**

- **Character-level rendering** with typing state information
- **Line-based rendering** with intelligent word wrapping
- **Cursor position tracking** across line boundaries
- **Unicode support** for international characters and emojis
- **Generic renderer interface** for any UI framework

### ⚙️ **Configurable Behavior**

- **Measurement intervals** for statistics collection
- **Line wrapping options** (word boundaries vs. character wrapping)
- **Newline handling** (respect or ignore paragraph breaks)
- **Performance tuning** for different use cases

## 🔗 Relationship to OctoType

Gladius serves as the **core engine** for
[OctoType](https://github.com/mahlquistj/octotype), a TUI typing trainer. While
OctoType provides the user interface, configuration system, and TUI experience,
Gladius handles all the fundamental typing logic:

- **Text processing and character management**
- **Real-time typing statistics calculation**
- **Input validation and error tracking**
- **Rendering pipeline for display**
- **Performance metrics and analysis**

This separation allows:

- **Reusability**: Other applications can use Gladius as a typing engine
- **Testing**: Core typing logic can be thoroughly tested independently
- **Maintainability**: Clear separation of concerns between UI and logic
- **Performance**: Optimized core without UI overhead

## 📦 Installation

Add Gladius to your `Cargo.toml`:

```toml
[dependencies]
gladius = "0.3.2"
```

## 📚 Documentation

Complete API documentation is available at
[docs.rs/gladius](https://docs.rs/gladius).

## 🧪 Examples

### Basic Typing Session

```rust
use gladius::{TypingSession, CharacterResult};

let mut session = TypingSession::new("The quick brown fox").unwrap();

// Process typing input
match session.input(Some('T')) {
    Some((ch, CharacterResult::Correct)) => println!("Correct: {}", ch),
    Some((ch, CharacterResult::Wrong)) => println!("Wrong: {}", ch),
    Some((ch, CharacterResult::Corrected)) => println!("Corrected: {}", ch),
    Some((ch, CharacterResult::Deleted(state))) => println!("Deleted: {} (was {:?})", ch, state),
    None => println!("No input processed"),
}
```

### Custom Configuration

```rust
use gladius::{TypingSession, config::Configuration};

let config = Configuration {
    measurement_interval_seconds: 0.5, // More frequent measurements
};

let session = TypingSession::new("Hello, world!")
    .unwrap()
    .with_configuration(config);
```

### Character-level Rendering

```rust
use gladius::TypingSession;

let session = TypingSession::new("hello").unwrap();

let rendered: Vec<String> = session.render(|ctx| {
    let cursor = if ctx.has_cursor { " |" } else { "" };
    let state = match ctx.character.state {
        gladius::State::Correct => "✓",
        gladius::State::Wrong => "✗",
        gladius::State::None => "·",
        _ => "?",
    };
    format!("{}{}{}", ctx.character.char, state, cursor)
});
```

## ⚡ Performance Characteristics

| Operation         | Time Complexity                   | Notes                                                      |
| ----------------- | --------------------------------- | ---------------------------------------------------------- |
| Character input   | O(1) amortized, O(w) worst case   | Usually constant, worst case when recalculating word state |
| Character lookup  | O(1)                              | Direct vector indexing                                     |
| Word lookup       | O(1)                              | Pre-computed mapping                                       |
| Statistics update | O(1) typical, O(m) when measuring | Most updates are constant, measurements scan history       |
| Rendering         | O(n)                              | Linear in text length                                      |

## 🛡️ Thread Safety

Gladius types are not thread-safe by design for maximum performance. Each typing
session should be used on a single thread. Multiple sessions can run
concurrently on different threads.

## 🔧 Minimum Supported Rust Version (MSRV)

Gladius supports Rust 1.88.0 and later.

## 🤝 Contributing

Gladius development happens alongside OctoType. Contributions are welcome!
Please see the [OctoType repository](https://github.com/mahlquistj/octotype) for
contribution guidelines.

## 📄 License

Licensed under the MIT License. See [LICENSE](../LICENSE) for details.

## Why "Gladius"?

Gladius is the Latin word for a small sword, but in biology, it's the name for
the internal, feather-shaped shell of a squid.

Since gladius is the **core** library of **Octo**Type, this name felt very
fitting.
