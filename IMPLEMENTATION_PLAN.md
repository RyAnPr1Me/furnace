# Comprehensive Feature Implementation Plan

All previously deleted features have been restored to config structures. Now they must be fully implemented.

## Status: In Progress

### Phase 1: Configuration Integration (✅ COMPLETE)
- [x] Restore all config structures
- [x] Restore parsing logic
- [x] Restore default implementations
- [x] Code compiles

### Phase 2: KeyBindings Integration (NEXT - HIGH PRIORITY)
**Goal**: Make Config.keybindings functional

**Implementation**:
1. Create key combination parser (parse "Ctrl+T" → KeyCode + Modifiers)
2. Integrate with KeybindingManager in Terminal::new()
3. Register config bindings after loading defaults
4. Allow config to override default bindings

**Files to modify**:
- `src/terminal/mod.rs`: Terminal::new() - register config bindings
- `src/keybindings.rs`: Add parse_key_combination() method
- Add method to KeybindingManager to register from config

**Estimated Complexity**: Medium (2-3 hours)

### Phase 3: Text Selection (HIGH PRIORITY)
**Goal**: Make ThemeConfig.selection functional

**Implementation**:
1. Add selection state to Terminal struct (start_pos, end_pos, active)
2. Handle mouse events (click and drag)
3. Render selection with custom color
4. Implement copy to clipboard (crossterm clipboard or arboard crate)

**Files to modify**:
- `src/terminal/mod.rs`: Add selection fields, mouse handler, render logic
- Use `ThemeConfig.selection` color for selection background

**Estimated Complexity**: High (4-6 hours)

### Phase 4: Lua Custom Keybindings (MEDIUM PRIORITY)
**Goal**: Make HooksConfig.custom_keybindings functional

**Implementation**:
1. Parse custom_keybindings map from config
2. Register with KeybindingManager as custom actions
3. On trigger, execute Lua function from map
4. Pass context (current directory, command, etc.) to Lua

**Files to modify**:
- `src/terminal/mod.rs`: Register custom bindings
- `src/hooks.rs`: Add execute_keybinding_function() method
- `src/keybindings.rs`: Support custom Lua actions

**Estimated Complexity**: Medium (3-4 hours)

### Phase 5: Lua Output Filters (MEDIUM PRIORITY)
**Goal**: Make HooksConfig.output_filters functional

**Implementation**:
1. Create filter pipeline in shell output processing
2. Each filter is a Lua function that transforms string → string
3. Apply filters in sequence before displaying output
4. Handle errors gracefully (log and skip filter)

**Files to modify**:
- `src/terminal/mod.rs`: Shell output reading - apply filters
- `src/hooks.rs`: Add apply_output_filters() method

**Estimated Complexity**: Medium (2-3 hours)

### Phase 6: Lua Custom Widgets (LOW PRIORITY - COMPLEX)
**Goal**: Make HooksConfig.custom_widgets functional

**Implementation**:
1. Define widget rendering API (Lua returns ratatui widget specs?)
2. Reserve screen space for custom widgets
3. Execute Lua widget code each frame
4. Render returned widget specifications

**Files to modify**:
- `src/terminal/mod.rs`: Render method - widget areas
- `src/hooks.rs`: Add execute_widget() method
- Define Lua → ratatui widget bridge

**Estimated Complexity**: Very High (6-8 hours) - requires careful API design

### Phase 7: Background Images (MEDIUM-HIGH PRIORITY)
**Goal**: Make BackgroundConfig functional

**Implementation**:
1. Load image file (use `image` crate - already in dependencies)
2. Implement rendering modes:
   - fill: scale to fill, crop excess
   - fit: scale to fit, maintain aspect
   - stretch: distort to fill
   - tile: repeat pattern
   - center: center without scaling
3. Apply opacity (alpha blending)
4. Apply blur effect (Gaussian blur from image crate)
5. Render as background layer before terminal content

**Files to modify**:
- `src/terminal/mod.rs`: Load and render background
- May require custom rendering layer or ratatui customization

**Estimated Complexity**: High (5-7 hours)
**Challenge**: Ratatui doesn't support background images natively - may need workaround

### Phase 8: Cursor Trails (LOW-MEDIUM PRIORITY)
**Goal**: Make CursorTrailConfig functional

**Implementation**:
1. Track cursor position history (circular buffer of positions)
2. Store timestamps for each position
3. Render trail positions with fading based on:
   - linear: constant fade rate
   - exponential: faster fade for older positions
   - smooth: ease-out curve
4. Apply color with alpha channel
5. Update trail every animation_speed ms

**Files to modify**:
- `src/terminal/mod.rs`: Cursor tracking, trail rendering
- Add trail state struct

**Estimated Complexity**: Medium-High (4-5 hours)
**Challenge**: May require custom rendering or overlays

## Priority Order for Implementation

1. **KeyBindings Integration** - Quick win, enables config customization
2. **Text Selection** - Important UX feature, uses selection color
3. **Lua Custom Keybindings** - Extends keybinding system
4. **Lua Output Filters** - Useful for power users
5. **Background Images** - Visual enhancement (complex, may defer)
6. **Cursor Trails** - Visual enhancement (nice-to-have)
7. **Lua Custom Widgets** - Most complex, defer to end

## Technical Challenges

### Challenge 1: Ratatui Limitations
- Ratatui is designed for TUI apps, not rich graphics
- Background images and cursor trails may require:
  - Custom rendering backend
  - Layering system
  - OR switching to GPU renderer for these features

**Solution Options**:
A. Implement in ratatui (hacky but works)
B. Make these features GPU-only (cleaner architecture)
C. Create hybrid renderer (ratatui + custom layer)

**Recommendation**: Option B - Make background/trails GPU-only, implement selection in ratatui

### Challenge 2: Thread Safety
- Lua execution must be thread-safe
- HooksExecutor already uses Lua instance
- Custom keybindings/filters/widgets share same executor

**Solution**: Ensure all Lua calls use proper locking/synchronization

### Challenge 3: Performance
- Output filters on every output chunk could be expensive
- Image loading/blur on every frame would be too slow

**Solution**:
- Cache processed images
- Debounce filter application
- Run heavy operations async

## Implementation Strategy

Given the scope, implement in phases with testing after each:

1. Start with KeyBindings (easiest, highest value)
2. Implement Text Selection (high value, moderate complexity)
3. Add Lua features one at a time
4. Visual features (background/trails) last or mark as GPU-only

## Decision Point: GPU vs Ratatui

For background images and cursor trails, we have two options:

**Option A: Implement in Ratatui**
- Pros: Works without GPU feature
- Cons: Hacky, limited, poor performance

**Option B: Make GPU-Only**
- Pros: Clean architecture, better performance
- Cons: Features not available without GPU

**Recommendation**: Make visual features GPU-only, update docs accordingly.
This keeps code clean and performant.

## Next Steps

1. Implement KeyBindings integration
2. Implement Text Selection
3. Implement Lua features
4. Document GPU-only features
5. Test thoroughly
6. Update DEAD_CODE_FIXES.md with implementation details
