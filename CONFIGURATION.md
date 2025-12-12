# Furnace configuration guide

This document lists every supported configuration field, their defaults, and examples for Lua scripting. Furnace loads a Lua file that defines a global `config` table; the file is executed, so only load trusted configs. The optional YAML example in the repo is illustrative only—the runtime reads **Lua**.

## Where the config is loaded from
- Default path: `~/.furnace/config.lua` (override with `furnace --config /path/to/config.lua`).
- The file must set `config = { ... }` at top level.
- Any field you omit falls back to the defaults below.

## Shell (defaults)
| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `default_shell` | string | auto-detected (`pwsh.exe` → `powershell.exe` → `cmd.exe` on Windows; `$SHELL` or `/bin/bash` on Unix) | Set to an explicit executable path or name in `PATH`. |
| `working_dir` | string or `nil` | `nil` (home directory) | Set to start new sessions in a specific directory. |
| `env` | table<string,string> | `{}` | Extra environment variables passed to the shell. |

## Terminal (defaults)
| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `max_history` | number | `10000` | Command history capacity (circular buffer). |
| `enable_tabs` | bool | `false` | Enable multi-tab UI. |
| `enable_split_pane` | bool | `false` | Enable horizontal/vertical splits. |
| `font_size` | number | `12` | Font size metadata. |
| `cursor_style` | string | `"block"` | One of `"block"`, `"underline"`, `"bar"`. |
| `scrollback_lines` | number | `10000` | Scrollback buffer length. |
| `hardware_acceleration` | bool | `true` | GPU if built with `--features gpu`, otherwise CPU fallback. |

## Theme (defaults)
| Field | Type | Default | Notes |
| --- | --- | --- | --- |
| `name` | string | `"default"` | Theme identifier. |
| `foreground` | string | `#FFFFFF` | |
| `background` | string | `#1E1E1E` | |
| `cursor` | string | `#00FF00` | |
| `selection` | string | `#264F78` | |
| `colors.*` | string | Normal: `black #000000`, `red #FF0000`, `green #00FF00`, `yellow #FFFF00`, `blue #0000FF`, `magenta #FF00FF`, `cyan #00FFFF`, `white #FFFFFF`; Bright: `bright_black #808080`, `bright_red #FF8080`, `bright_green #80FF80`, `bright_yellow #FFFF80`, `bright_blue #8080FF`, `bright_magenta #FF80FF`, `bright_cyan #80FFFF`, `bright_white #FFFFFF` | ANSI palette. |

### Optional theme extensions
- `background_image` (table, ignored if both `image_path` and `color` are absent)
  - `image_path`: path to an image file.
  - `color`: fallback solid color (also used alone for a solid background).
  - `opacity`: default `1.0`.
  - `mode`: default `"fill"` (`fill` | `fit` | `stretch` | `tile` | `center`).
  - `blur`: default `0.0`.
- `cursor_trail` (table, optional)
  - `enabled`: default `false`.
  - `length`: default `10`.
  - `color`: default `"#00FF0080"` (supports alpha).
  - `fade_mode`: default `"exponential"` (`linear` | `exponential` | `smooth`).
  - `width`: default `1.0`.
  - `animation_speed`: default `16` (ms).

## Keybindings (defaults)
| Action | Default |
| --- | --- |
| `new_tab` | `Ctrl+T` |
| `close_tab` | `Ctrl+W` |
| `next_tab` | `Ctrl+Tab` |
| `prev_tab` | `Ctrl+Shift+Tab` |
| `split_vertical` | `Ctrl+Shift+V` |
| `split_horizontal` | `Ctrl+Shift+H` |
| `copy` | `Ctrl+Shift+C` |
| `paste` | `Ctrl+Shift+V` |
| `search` | `Ctrl+F` |
| `clear` | `Ctrl+L` |

> `split_vertical` conflicts with the default `paste` binding. Rebind `split_vertical` (e.g., `Ctrl+Shift+|` or `Ctrl+Alt+V`) if you enable splits.

## Features (all default to `false`)
- `resource_monitor`
- `autocomplete`
- `progress_bar`
- `session_manager`
- `theme_manager`
- `command_palette`

## Hooks (all optional)
All fields in this section live under `config.hooks`. Lifecycle hooks accept Lua code (string path or inline code):
- `on_startup`
- `on_shutdown`
- `on_key_press`
- `on_command_start`
- `on_command_end`
- `on_output`
- `on_bell`
- `on_title_change`

Other extensibility (also inside `config.hooks`):
- `custom_keybindings`: map of key → Lua function (string).
- `output_filters`: array of Lua functions that transform terminal output.
- `custom_widgets`: array of Lua snippets to render extra UI elements.

## Minimal config example
```lua
config = {
    shell = { default_shell = "/bin/bash" },
    terminal = { enable_tabs = true },
    features = { progress_bar = true }
}
```

## Split-friendly keybinding example
```lua
config = {
    terminal = { enable_split_pane = true },
    keybindings = {
        split_vertical = "Ctrl+Alt+V",
        paste = "Ctrl+Shift+V"
    }
}
```

## Hooks and output filters example
```lua
config = {
    hooks = {
        on_command_end = [[
            if exit_code ~= 0 then
                print("Command failed: " .. command)
            end
        ]],
        output_filters = {
            "function(text) return text:gsub('ERROR', '🔴 ERROR') end",
            "function(text) return text:gsub('SUCCESS', '✅ SUCCESS') end"
        },
        custom_keybindings = {
            ["Ctrl+Shift+G"] = "function() print(os.date()) end"
        }
    }
}
```

## Lua scripting tips
- The config file is regular Lua: you can compute values, require other Lua files, or inspect environment variables.
- Keep expensive logic out of hot paths; hooks run in response to events.
- When binding functions (e.g., in `custom_keybindings`), provide pure Lua code as strings. Example:
  ```lua
  config.hooks.custom_keybindings = {
      ["Ctrl+Shift+B"] = [[
          function()
              os.execute("cargo build")
          end
      ]]
  }
  ```
- Always ensure your script returns quickly to avoid UI stalls.

## Security considerations
- The config file is executed with full Lua capabilities (file I/O, OS access). Only load trusted configs.
- Avoid committing secrets inside hooks or environment settings.
