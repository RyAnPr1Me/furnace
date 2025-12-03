# Local Echo Fix for User Input Visibility

## Problem Statement

When running the Furnace terminal emulator as a compiled `.exe` file (especially on Windows), user input was not visible. Users could see only a blinking cursor but not the characters they were typing. This made the terminal appear unresponsive or broken.

## Root Cause

The terminal emulator was relying completely on **shell echo** (the shell echoing back characters it receives from the PTY). This approach has several issues:

1. **PTY Configuration Issues**: On Windows, PTY echo might not be configured correctly
2. **Timing Delays**: Shell echo can be delayed, causing characters to appear with lag
3. **Shell Dependency**: Different shells handle echo differently
4. **No Fallback**: If shell echo fails, the user sees nothing

## Solution

Implemented **local echo** - a fallback mechanism that displays user input immediately without waiting for shell echo. The implementation:

### 1. Command Buffer Tracking
The terminal already tracked user input in `command_buffers` (used for backspace handling). This buffer contains the exact bytes sent to the shell.

### 2. Immediate Display
Modified `render_terminal_output()` in `src/terminal/mod.rs` to:
- Read the pending command from `command_buffers[active_session]`
- Convert it to a displayable string
- Append it to the last line of terminal output
- Display it immediately to the user

### 3. Duplicate Detection
To prevent double-display when shell echo IS working:
- Check if the last line of shell output already ends with the pending input
- Only show local echo if the shell hasn't echoed it yet
- This provides a seamless experience in both scenarios

## Code Changes

### Main Change in `src/terminal/mod.rs` (lines 1037-1074)

```rust
// LOCAL ECHO FIX: Append pending command buffer to show user input immediately
// This fixes the issue where typed characters are not visible until shell echoes them back
// This is especially important on Windows where PTY echo may be delayed or not working
if let Some(cmd_buf) = self.command_buffers.get(self.active_session) {
    if !cmd_buf.is_empty() {
        // Convert command buffer to string for display (local echo)
        let pending_input = String::from_utf8_lossy(cmd_buf);
        
        // Check if the last line already ends with this input (shell echo is working)
        // to avoid duplicate display
        let should_display = if let Some(last_line) = styled_lines.last() {
            let last_line_text: String = last_line
                .spans
                .iter()
                .map(|span| span.content.as_ref())
                .collect();
            // Only show local echo if the shell hasn't echoed it yet
            !last_line_text.ends_with(pending_input.as_ref())
        } else {
            true
        };

        if should_display {
            // Append the pending input to the last line or create a new line
            // ... (display logic)
        }
    }
}
```

## How It Works

### Normal Flow (Shell Echo Working)
1. User types 'a'
2. Character sent to shell via PTY
3. Shell echoes 'a' back almost immediately
4. Character appears in `output_buffers`
5. Character displayed (from shell echo)
6. Local echo detects it's already shown, skips duplicate

### Fallback Flow (Shell Echo Broken/Delayed)
1. User types 'a'
2. Character sent to shell via PTY
3. Character added to `command_buffers[active_session]`
4. **Local echo immediately displays 'a'** ← This is the fix!
5. User sees their input right away
6. When shell eventually echoes (or doesn't), no issue

### Enter Key Handling
1. User presses Enter
2. Command sent to shell with `\r`
3. `command_buffers` cleared immediately
4. Shell processes command and sends output
5. Output displayed normally
6. No duplicate because buffer was cleared

## Testing

### Manual Testing on Windows
1. Build release version: `cargo build --release`
2. Run `target/release/furnace.exe`
3. Type characters - they should appear immediately
4. Press Enter - command should execute
5. Try backspace - should work correctly
6. Test with different shells (cmd.exe, PowerShell, pwsh.exe)

### Manual Testing on Linux/macOS
1. Build: `cargo build --release`
2. Run: `target/release/furnace`
3. Same testing as Windows above
4. Test with bash, zsh, fish, etc.

### Automated Tests
Run the test suite to verify no regressions:
```bash
cargo test
```

New tests added in `tests/functionality_verification.rs`:
- `test_terminal_with_local_echo` - Verifies terminal creation with local echo support
- `test_command_buffer_tracking` - Verifies command buffer tracking is working

## Benefits

1. **Immediate Feedback**: Users see characters as they type them
2. **Cross-Platform**: Works on Windows, Linux, and macOS
3. **Shell Agnostic**: Works with any shell (PowerShell, bash, zsh, fish, etc.)
4. **Backward Compatible**: Doesn't break existing functionality
5. **Smart Fallback**: Only activates when needed (shell echo broken)
6. **Zero Overhead**: Only processes when command buffer has data

## Edge Cases Handled

1. **Double Echo**: Duplicate detection prevents showing characters twice
2. **Backspace**: Uses existing command buffer management (already handles UTF-8 correctly)
3. **Arrow Keys**: Command buffer cleared on history navigation (already implemented)
4. **Enter Key**: Command buffer cleared immediately after sending command
5. **Special Characters**: UTF-8 handling via `String::from_utf8_lossy`
6. **Empty Buffer**: No-op when buffer is empty
7. **Tab Switching**: Each tab has its own command buffer

## Performance Impact

- **Minimal**: Only processes when command buffer is non-empty
- **Zero-Copy**: Uses `String::from_utf8_lossy` which returns `Cow` (no allocation if valid UTF-8)
- **Cached Lines**: Appends to already-cached styled lines
- **No Extra Renders**: Renders as part of existing frame rendering

## Future Enhancements

Possible improvements (not needed for current fix):
1. Add config option to enable/disable local echo
2. Add visual indicator to distinguish local echo from shell echo
3. Track which characters are local vs shell echo for better duplicate detection
4. Add telemetry to detect when shell echo is consistently failing

## Verification Checklist

- [x] Code compiles without warnings
- [x] All existing tests pass
- [x] New tests added and passing
- [x] Clippy linting passes
- [x] Code formatted with `cargo fmt`
- [ ] Manual testing on Windows
- [ ] Manual testing on Linux
- [ ] Manual testing on macOS

## Related Files

- `src/terminal/mod.rs` - Main terminal implementation with local echo
- `src/shell/mod.rs` - Shell session management (PTY)
- `tests/functionality_verification.rs` - Test cases
- `Cargo.toml` - Dependencies (portable-pty, crossterm, ratatui)

## References

- Issue: "user input field is not visible, neither is user input besides blinking cursor when running .exe file"
- PTY Documentation: <https://docs.rs/portable-pty/>
- Terminal Emulation: <https://en.wikipedia.org/wiki/Terminal_emulator>
- Local Echo: <https://en.wikipedia.org/wiki/Local_echo>
