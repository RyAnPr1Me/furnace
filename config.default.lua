-- ============================================================================
-- Furnace Terminal Emulator - Default Configuration
-- ============================================================================
-- Copy to: ~/.furnace/config.lua (or use --config /path/to/config.default.lua)
-- This file mirrors Furnace's built-in defaults with sensible, minimal options.
-- ============================================================================

config = {
    shell = {
        -- Leave nil to auto-detect ($SHELL on Unix; pwsh/powershell/cmd on Windows)
        default_shell = nil,
        -- Starting directory (nil = home)
        working_dir = nil,
        -- Extra environment variables
        env = {
            -- MY_VAR = "value",
        },
    },

    terminal = {
        max_history = 10000,
        enable_tabs = false,
        enable_split_pane = false,
        font_size = 12,
        cursor_style = "block", -- "block" | "underline" | "bar"
        scrollback_lines = 10000,
        hardware_acceleration = true, -- uses GPU if built with `--features gpu`, else CPU fallback
    },

    theme = {
        name = "default",
        foreground = "#FFFFFF",
        background = "#1E1E1E",
        cursor = "#00FF00",
        selection = "#264F78",
        colors = {
            -- Normal
            black = "#000000",
            red = "#FF0000",
            green = "#00FF00",
            yellow = "#FFFF00",
            blue = "#0000FF",
            magenta = "#FF00FF",
            cyan = "#00FFFF",
            white = "#FFFFFF",
            -- Bright
            bright_black = "#808080",
            bright_red = "#FF8080",
            bright_green = "#80FF80",
            bright_yellow = "#FFFF80",
            bright_blue = "#8080FF",
            bright_magenta = "#FF80FF",
            bright_cyan = "#80FFFF",
            bright_white = "#FFFFFF",
        },
    },

    features = {
        resource_monitor = false,
        autocomplete = false,
        progress_bar = false,
        session_manager = false,
        theme_manager = false,
        command_palette = false,
    },

    keybindings = {
        new_tab = "Ctrl+T",
        close_tab = "Ctrl+W",
        next_tab = "Ctrl+Tab",
        prev_tab = "Ctrl+Shift+Tab",
        split_vertical = "Ctrl+Shift+V",
        split_horizontal = "Ctrl+Shift+H",
        copy = "Ctrl+Shift+C",
        paste = "Ctrl+Shift+V",
        search = "Ctrl+F",
        clear = "Ctrl+L",
    },

    hooks = {
        on_startup = nil,
        on_shutdown = nil,
        on_key_press = nil,
        on_command_start = nil,
        on_command_end = nil,
        on_output = nil,
        on_bell = nil,
        on_title_change = nil,
        custom_keybindings = {},
        output_filters = {},
        custom_widgets = {},
    },
}
