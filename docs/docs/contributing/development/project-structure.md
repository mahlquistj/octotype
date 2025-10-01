---
title: Project Structure
---

# OctoType Project Structure

This guide provides a comprehensive overview of the OctoType project
architecture for contributors. The project follows a modular design with clear
separation between the core typing engine library (Gladius) and the terminal
user interface application (OctoType).

## Repository Overview

OctoType is structured as a Rust workspace containing two main crates:

```mermaid
graph TB
    A[octotype/] --> B[Gladius Library]
    A --> C[OctoType TUI Application]
    A --> D[Documentation]
    A --> E[Configuration Files]
    
    B --> B1[Core Typing Engine]
    B --> B2[Statistics & Analytics]
    B --> B3[Rendering System]
    
    C --> C1[Terminal Interface]
    C --> C2[Configuration Management]
    C --> C3[Page/UI Components]
    
    D --> D1[User Documentation]
    D --> D2[API Documentation]
    D --> D3[Contributing Guides]
```

## Core Components

### 1. Gladius Library (`gladius/`)

Gladius is the high-performance typing trainer library that provides the core
functionality for any typing trainer application.

```mermaid
graph LR
    subgraph "Gladius Core Architecture"
        A[Session] --> B[Buffer]
        A --> C[Input Handler]
        A --> D[Statistics Tracker]
        
        B --> E[Render System]
        C --> F[Statistics]
        D --> F
        
        G[Math Utilities] --> F
        G --> D
        
        H[Config] --> A
    end
```

**Key modules:**

- `session.rs` - Main typing session orchestration
- `buffer.rs` - Text buffer and character state management
- `input_handler.rs` - User input processing and validation
- `statistics_tracker.rs` - Real-time performance metrics
- `statistics.rs` - Data structures for typing statistics
- `render.rs` - Generic rendering interface
- `math.rs` - Mathematical utilities and algorithms
- `config.rs` - Library configuration

**Performance Features:**

- O(1) keystroke handling
- Efficient character-to-word mapping
- Welford's algorithm for numerical stability
- Minimal memory allocations during typing

### 2. OctoType Application (`octotype/`)

The terminal-based typing trainer application built on top of Gladius.

```mermaid
graph TD
    subgraph "OctoType Application Architecture"
        A[main.rs] --> B[App]
        B --> C[Page Router]
        
        C --> D[Menu Page]
        C --> E[Session Page]
        C --> F[Stats Page]
        C --> G[Error Page]
        C --> H[Loadscreen Page]
        
        I[Config System] --> B
        I --> J[Theme Config]
        I --> K[Mode Config]
        I --> L[Source Config]
        I --> M[Parameters Config]
        I --> N[Stats Config]
        
        O[Utils] --> B
        
        E --> P[Session Modes]
        E --> Q[Text Sources]
    end
```

**Key modules:**

#### Application Core

- `main.rs` - Entry point and CLI argument parsing
- `app.rs` - Main application loop and event handling
- `page.rs` - Page routing and state management
- `utils.rs` - Utility functions and constants

#### Configuration System (`config/`)

- `config.rs` - Main configuration orchestration
- `theme.rs` - Color themes and visual styling
- `mode.rs` - Typing modes (time-based, word-based, etc.)
- `source.rs` - Text sources configuration
- `parameters.rs` - Session parameters and settings
- `stats.rs` - Statistics display configuration

#### Page System (`page/`)

- `menu.rs` - Main menu interface
- `session.rs` - Active typing session interface
- `stats.rs` - Performance statistics display
- `error.rs` - Error handling and display
- `loadscreen.rs` - Loading states and transitions

#### Session Components (`page/session/`)

- `mode.rs` - Session mode implementations
- `text.rs` - Text processing and management

## Data Flow

```mermaid
sequenceDiagram
    participant User
    participant App
    participant Page
    participant Gladius
    participant Config
    
    User->>App: Input Event
    App->>Page: Route Event
    Page->>Config: Load Settings
    Page->>Gladius: Create Session
    
    loop Typing Session
        User->>App: Keystroke
        App->>Page: Handle Input
        Page->>Gladius: Process Character
        Gladius->>Gladius: Update Statistics
        Gladius->>Page: Return Result
        Page->>App: Update UI
        App->>User: Render Changes
    end
    
    Page->>Gladius: Get Final Stats
    Gladius->>Page: Return Statistics
    Page->>App: Show Results
```

## Configuration Architecture

The configuration system supports multiple file formats and sources:

```mermaid
graph LR
    A[Environment Variables] --> D[Figment Config Merger]
    B[TOML Files] --> D
    C[CLI Arguments] --> D
    
    D --> E[Unified Config]
    
    E --> F[Theme Settings]
    E --> G[Mode Definitions]
    E --> H[Text Sources]
    E --> I[Global Parameters]
```

Configuration files are typically located in:

- `~/.config/octotype/` (Linux/macOS)
- `%APPDATA%/octotype/` (Windows)

## Build System

The project uses Cargo workspaces for efficient dependency management:

```mermaid
graph TB
    A[Cargo.toml<br/>Workspace Root] --> B[gladius/Cargo.toml]
    A --> C[octotype/Cargo.toml]
    
    B --> D[Gladius Dependencies]
    C --> E[OctoType Dependencies]
    C --> F[gladius path dependency]
    
    D --> G[web-time, simple-mermaid]
    E --> H[ratatui, crossterm, clap, serde, figment]
```

**Key Build Features:**

- Optimized release builds with LTO and strip
- Shared workspace dependencies
- Criterion benchmarks for performance testing
- Clippy linting with strict rules

## Documentation System

Documentation is built using Docusaurus and includes:

```
docs/
├── docs/
│   ├── configuration/          # User configuration guides
│   └── contributing/           # Contributor documentation
│       └── development/        # Development guides
├── src/                        # Docusaurus source
└── docusaurus.config.ts        # Docusaurus configuration
```

## Performance Considerations

- **Gladius** prioritizes performance for real-time typing analysis
- **OctoType** balances performance with user experience
- Critical paths are optimized for minimal latency
- Statistics calculations use numerically stable algorithms
- Memory allocations are minimized during active typing

This architecture enables both standalone library usage and a complete typing
trainer application while maintaining clear separation of concerns and high
performance.
