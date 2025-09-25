# WPM/Stats and Text Engine Separation Plan

## Current State Analysis

**Current Coupling Points:**
- `page/session.rs`: Contains both WPM calculation (`calculate_wpm:233`, `calculate_acc:253`) and rendering logic (`render:343`)
- `page/session/text.rs`: `Segment` handles both text state/input tracking AND rendering (`render_line:157`)
- `page/session/stats.rs`: Stats collection and chart rendering are mixed

**Key Issues:**
1. Stats calculations are embedded in the main session logic
2. Text segments handle both data and presentation
3. No clear separation between text engine and stats engine
4. Inflexible text rendering (hard to add code blocks, etc.)

## Proposed Architecture

### 1. **Stats Engine Module** (`src/stats/`)
```
src/stats/
├── mod.rs           # Public API
├── engine.rs        # Core stats calculation engine
├── metrics.rs       # WPM, accuracy, consistency calculations  
├── collector.rs     # Data collection and aggregation
└── types.rs         # Shared types (Wpm, Timestamp, etc.)
```

### 2. **Text Engine Module** (`src/text/`)
```
src/text/
├── mod.rs           # Public API
├── engine.rs        # Core text processing engine
├── segment.rs       # Text segment data model (no rendering)
├── input.rs         # Input handling and validation
├── renderer/        # Rendering subsystem
│   ├── mod.rs       # Renderer trait and factory
│   ├── basic.rs     # Current simple text renderer
│   └── code.rs      # Code block renderer (extensible)
└── types.rs         # Text-related types
```

### 3. **Updated Session Module** (`src/page/session/`)
```
src/page/session/
├── mod.rs           # Orchestrates text and stats engines
├── mode.rs          # (unchanged)
└── stats.rs         # Only UI rendering, no calculations
```

## Detailed Implementation Plan

### **Phase 1: Extract Stats Engine**
1. **Create `src/stats/types.rs`**
   - Move `Wpm`, `RunningStats` from `session/stats.rs`
   - Add `StatsEvent` enum for decoupled communication

2. **Create `src/stats/engine.rs`**
   - Extract `calculate_wpm()`, `calculate_acc()` from `session.rs:233,253`
   - Create `StatsEngine` struct with pure calculation methods
   - No UI dependencies

3. **Create `src/stats/collector.rs`**  
   - Extract stats collection logic from `session.rs:130` (`update_stats`)
   - Handle error tracking, timing, aggregation

### **Phase 2: Extract Text Engine**
1. **Create `src/text/segment.rs`**
   - Move core `Segment` data model from `session/text.rs`
   - Remove `render_line()` method - keep only data operations
   - Focus on: input validation, character tracking, word boundaries

2. **Create `src/text/engine.rs`**
   - Extract `text_to_segments()` from `session.rs:303`
   - Add support for different text types (plain, code, markdown)
   - Implement flexible segment creation

3. **Create `src/text/renderer/mod.rs`**
   - Define `TextRenderer` trait:
     ```rust
     trait TextRenderer {
         fn render_segment(&self, segment: &Segment, config: &RenderConfig) -> Line;
         fn supports_type(&self, text_type: TextType) -> bool;
     }
     ```

4. **Create `src/text/renderer/basic.rs`**
   - Move `render_line()` logic from `text.rs:157`
   - Implement `TextRenderer` for current functionality

### **Phase 3: Update Session Orchestration**
1. **Refactor `session.rs`**
   - Replace direct stats calculations with `StatsEngine` calls
   - Replace text handling with `TextEngine` calls  
   - Session becomes orchestrator: handles events, coordinates engines
   - Remove: `calculate_wpm`, `calculate_acc`, `update_stats`, `text_to_segments`

2. **Update `session/stats.rs`**
   - Keep only UI rendering logic
   - Remove calculation methods  
   - Receive pre-calculated stats from `StatsEngine`

### **Phase 4: Enable Extensible Text Rendering**
1. **Add `src/text/renderer/code.rs`**
   - Implement syntax highlighting for code blocks
   - Support different programming languages
   - Handle indentation and special characters

2. **Create renderer factory in `text/renderer/mod.rs`**
   - Auto-detect text type (plain, code, markdown)
   - Route to appropriate renderer
   - Allow runtime renderer registration

## Benefits of This Architecture

1. **Separation of Concerns**: Stats calculations completely independent of rendering
2. **Testability**: Both engines can be unit tested in isolation  
3. **Extensibility**: Easy to add new text renderers (code, markdown, etc.)
4. **Reusability**: Stats engine could be used in other contexts
5. **Maintainability**: Clear boundaries reduce complexity
6. **Performance**: Engines can be optimized independently

## Migration Strategy

1. **Backward Compatibility**: Keep existing APIs during transition
2. **Incremental**: Implement new modules alongside existing code
3. **Feature Flags**: Allow switching between old/new implementations
4. **Testing**: Ensure identical behavior during migration

## Key Files to Modify

### Current Files
- `src/page/session.rs` - Remove stats calculations, keep orchestration
- `src/page/session/text.rs` - Extract data model, remove rendering
- `src/page/session/stats.rs` - Keep only UI rendering

### New Files to Create
- `src/stats/` module (engine, types, collector, metrics)
- `src/text/` module (engine, segment, renderer system)
- Renderer implementations (basic, code, future: markdown)

## Implementation Order

1. **Stats Engine** - Extract and isolate all WPM/accuracy calculations
2. **Text Engine** - Separate text processing from rendering
3. **Renderer System** - Create extensible rendering architecture
4. **Session Refactor** - Update session to use new engines
5. **Enhanced Renderers** - Add code block and other specialized renderers

This modular approach ensures clean separation of concerns while enabling future extensibility for various text formats and rendering styles.