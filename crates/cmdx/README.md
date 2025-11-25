# cmdx

Cross-platform command translation library for Rust. Automatically translate Linux commands to Windows equivalents and vice versa.

[![Crates.io](https://img.shields.io/crates/v/cmdx.svg)](https://crates.io/crates/cmdx)
[![Documentation](https://docs.rs/cmdx/badge.svg)](https://docs.rs/cmdx)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **50+ command mappings** - Comprehensive coverage of common commands
- **Flag translation** - Intelligent argument and flag conversion
- **Pipeline support** - Handles pipes (`|`), redirects (`>`, `<`, `>>`), and command chaining (`&&`, `||`, `;`)
- **Zero-copy design** - Efficient static maps with minimal allocations
- **Error handling** - Detailed error types for translation failures
- **Configurable** - Enable/disable translation at runtime

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cmdx = "0.1"
```

## Quick Start

```rust
use cmdx::CommandTranslator;

fn main() {
    let translator = CommandTranslator::new(true);
    
    // On Linux, this translates Windows commands to Linux
    // On Windows, this translates Linux commands to Windows
    let result = translator.translate("ls -la /home");
    
    if result.translated {
        println!("Original: {}", result.original_command);
        println!("Translated: {}", result.final_command);
        println!("Description: {}", result.description);
    }
}
```

## Supported Commands

### File System
| Linux | Windows | Description |
|-------|---------|-------------|
| `ls` | `dir` | List directory contents |
| `cat` | `type` | Display file contents |
| `cp` | `copy`/`xcopy` | Copy files |
| `mv` | `move` | Move/rename files |
| `rm` | `del` | Remove files |
| `mkdir` | `md` | Create directories |
| `rmdir` | `rd` | Remove directories |
| `touch` | `type nul >` | Create empty files |
| `ln` | `mklink` | Create symbolic links |

### Text Processing
| Linux | Windows | Description |
|-------|---------|-------------|
| `grep` | `findstr` | Search text patterns |
| `head` | `Get-Content -Head` | Display first lines |
| `tail` | `Get-Content -Tail` | Display last lines |
| `sort` | `sort` | Sort lines |
| `diff` | `fc` | Compare files |

### System
| Linux | Windows | Description |
|-------|---------|-------------|
| `pwd` | `cd` | Print working directory |
| `clear` | `cls` | Clear screen |
| `which` | `where` | Locate command |
| `ps` | `tasklist` | List processes |
| `kill` | `taskkill` | Terminate process |

### Network
| Linux | Windows | Description |
|-------|---------|-------------|
| `ping` | `ping` | Ping host (flag translation) |
| `curl` | `Invoke-WebRequest` | HTTP requests |
| `ifconfig` | `ipconfig` | Network configuration |
| `traceroute` | `tracert` | Trace route |

## Pipeline Support

cmdx supports complex command pipelines:

```rust
use cmdx::CommandTranslator;

let translator = CommandTranslator::new(true);

// Pipes
let result = translator.translate("ls | grep foo");
assert!(result.has_pipeline);

// Redirects
let result = translator.translate("echo hello > output.txt");
assert!(result.has_pipeline);

// Command chaining
let result = translator.translate("mkdir test && cd test");
assert!(result.has_pipeline);
```

## Error Handling

```rust
use cmdx::{CommandTranslator, TranslationError};

let translator = CommandTranslator::new(true);
let result = translator.translate("some_command");

for error in &result.errors {
    match error {
        TranslationError::UnknownCommand(cmd) => {
            println!("Unknown command: {}", cmd);
        }
        TranslationError::InvalidSyntax(msg) => {
            println!("Invalid syntax: {}", msg);
        }
        _ => {}
    }
}
```

## Flag Translation Examples

### `ls` to `dir`
```
ls -a        → dir /A
ls -R        → dir /S
ls -1        → dir /B
ls -la /home → dir /A /home
```

### `rm` to `del`
```
rm -r folder  → del /S folder
rm -f file    → del /F /Q file
rm -rf folder → del /S /F /Q folder
```

### `grep` to `findstr`
```
grep -i pattern  → findstr /I pattern
grep -n pattern  → findstr /N pattern
grep -r pattern  → findstr /S pattern
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please see the [CONTRIBUTING](CONTRIBUTING.md) guide.

## Part of Furnace

cmdx is extracted from [Furnace](https://github.com/RyAnPr1Me/furnace), an advanced terminal emulator for Windows.
