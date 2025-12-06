-- ============================================================================
-- Furnace Terminal Emulator - Base Configuration
-- ============================================================================
-- A modern, high-performance terminal emulator built with Rust
-- This is a ready-to-use base configuration with sensible defaults
-- 
-- Installation:
--   1. Copy this file to: ~/.furnace/config.lua (Linux/macOS)
--                     or: %USERPROFILE%\.furnace\config.lua (Windows)
--   2. Customize as needed
--   3. Restart Furnace or reload configuration
--
-- Documentation: https://github.com/RyAnPr1Me/furnace
-- ============================================================================

config = {
    -- ========================================================================
    -- SHELL CONFIGURATION
    -- ========================================================================
    shell = {
        -- Default shell (auto-detected if not set)
        -- Windows examples: "pwsh.exe", "powershell.exe", "cmd.exe"
        -- Unix examples: "/bin/bash", "/bin/zsh", "/bin/fish"
        -- Leave as nil for auto-detection based on $SHELL or system default
        default_shell = nil,
        
        -- Starting directory (nil = use home directory)
        -- Examples: "~", "~/projects", "C:\\Users\\YourName\\code"
        working_dir = nil,
        
        -- Custom environment variables
        -- These are set before the shell starts
        env = {
            -- Example: Add a custom path
            -- MY_VAR = "value",
        }
    },

    -- ========================================================================
    -- TERMINAL BEHAVIOR
    -- ========================================================================
    terminal = {
        -- Command history size (uses efficient circular buffer)
        -- Recommended: 5000-10000 for typical usage
        max_history = 10000,
        
        -- Enable tabbed interface
        -- Set to true if you frequently work with multiple sessions
        enable_tabs = true,
        
        -- Enable split panes (horizontal and vertical)
        -- Set to true for side-by-side terminal views
        enable_split_pane = false,
        
        -- Font configuration
        font_size = 12,  -- Adjust based on your display and preferences
        
        -- Cursor appearance: "block", "underline", or "bar"
        cursor_style = "block",
        
        -- Scrollback buffer (lines of history to keep)
        -- Higher values use more memory but preserve more history
        -- Recommended: 10000 for typical usage, 50000+ for extensive logging
        scrollback_lines = 10000,
        
        -- Hardware acceleration (GPU rendering at 170 FPS)
        -- Highly recommended for smooth scrolling and responsiveness
        -- Disable only on systems without GPU support
        hardware_acceleration = true
    },

    -- ========================================================================
    -- THEME AND COLORS
    -- ========================================================================
    theme = {
        -- Theme identifier
        name = "furnace-dark",
        
        -- Core colors
        foreground = "#E0E0E0",  -- Main text color
        background = "#1A1A1A",  -- Background color (dark gray)
        cursor = "#00FF00",      -- Cursor color (bright green)
        selection = "#3A4A5A",   -- Text selection background
        
        -- ANSI color palette (24-bit true color)
        -- These are used for syntax highlighting and terminal applications
        colors = {
            -- Standard colors
            black = "#000000",
            red = "#E06C75",
            green = "#98C379",
            yellow = "#E5C07B",
            blue = "#61AFEF",
            magenta = "#C678DD",
            cyan = "#56B6C2",
            white = "#ABB2BF",
            
            -- Bright variants (used with bold attribute)
            bright_black = "#5C6370",
            bright_red = "#FF7B86",
            bright_green = "#A9D48A",
            bright_yellow = "#F6D18C",
            bright_blue = "#72C0FF",
            bright_magenta = "#D789EE",
            bright_cyan = "#67C7D3",
            bright_white = "#FFFFFF"
        }
    },

    -- ========================================================================
    -- OPTIONAL FEATURES
    -- ========================================================================
    -- These features are disabled by default for minimal resource usage
    -- Enable only the features you need
    features = {
        -- Resource monitor (Ctrl+R) - displays CPU, memory, and network usage
        -- Useful for monitoring system performance while working
        resource_monitor = false,
        
        -- Command autocomplete - suggests commands as you type
        -- Based on history and common commands
        autocomplete = false,
        
        -- Progress bar - visual indicator for long-running commands
        -- Automatically detects command execution
        progress_bar = true,
        
        -- Session manager - save and restore terminal sessions
        -- Preserve your work across restarts
        session_manager = false,
        
        -- Theme manager - switch themes dynamically
        -- Cycle through multiple color schemes
        theme_manager = false
    },

    -- ========================================================================
    -- KEYBOARD SHORTCUTS
    -- ========================================================================
    keybindings = {
        -- Tab management
        new_tab = "Ctrl+T",
        close_tab = "Ctrl+W",
        next_tab = "Ctrl+Tab",
        prev_tab = "Ctrl+Shift+Tab",
        
        -- Split pane management
        split_vertical = "Ctrl+Shift+V",
        split_horizontal = "Ctrl+Shift+H",
        
        -- Editing
        copy = "Ctrl+Shift+C",
        paste = "Ctrl+Shift+V",
        
        -- Navigation
        search = "Ctrl+F",
        clear = "Ctrl+L"
    }
}

-- ============================================================================
-- ADVANCED CONFIGURATION (Optional)
-- ============================================================================
-- Uncomment and customize these sections for advanced features

-- ----------------------------------------------------------------------------
-- Background Image (Optional)
-- ----------------------------------------------------------------------------
-- Add a custom background image for aesthetic appeal
-- Uncomment to enable:
--[[
config.theme.background_image = {
    image_path = "~/.furnace/backgrounds/wallpaper.png",
    opacity = 0.2,        -- 0.0 (transparent) to 1.0 (opaque)
    mode = "fill",        -- Options: "fill", "fit", "stretch", "tile", "center"
    blur = 3.0,           -- Blur strength (0.0 = no blur)
    color = "#1A1A1A"     -- Fallback solid color
}
]]

-- ----------------------------------------------------------------------------
-- Cursor Trail Effect (Optional)
-- ----------------------------------------------------------------------------
-- Add a smooth cursor trail for visual feedback
-- Uncomment to enable:
--[[
config.theme.cursor_trail = {
    enabled = true,
    length = 12,                  -- Number of trail segments (5-20 recommended)
    color = "#00FF0080",         -- Trail color with alpha (#RRGGBBAA)
    fade_mode = "exponential",   -- Options: "linear", "exponential", "smooth"
    width = 1.0,                 -- Trail width multiplier
    animation_speed = 16         -- Milliseconds per frame (~60 FPS)
}
]]

-- ----------------------------------------------------------------------------
-- Runtime Hooks (Advanced Extensibility)
-- ----------------------------------------------------------------------------
-- Execute custom Lua code at specific lifecycle events
-- This makes Furnace more extensible than any other terminal emulator
--
-- Uncomment and customize these hooks as needed:
--[[
config.hooks = {
    -- Lifecycle hooks
    on_startup = "~/.furnace/scripts/startup.lua",
    on_shutdown = "~/.furnace/scripts/cleanup.lua",
    
    -- Command tracking
    on_command_start = "~/.furnace/scripts/track_command.lua",
    on_command_end = "~/.furnace/scripts/command_stats.lua",
    
    -- Output processing
    on_output = "~/.furnace/scripts/highlight_errors.lua",
    
    -- Custom keybindings with Lua functions
    custom_keybindings = {
        ["Ctrl+Shift+G"] = [[
            function()
                local branch = io.popen("git branch --show-current 2>/dev/null"):read()
                if branch then
                    print("Git branch: " .. branch)
                else
                    print("Not a git repository")
                end
            end
        ]],
    },
    
    -- Output filters - transform text before display
    output_filters = {
        -- Highlight errors
        "function(text) return text:gsub('ERROR', '🔴 ERROR') end",
        -- Highlight success
        "function(text) return text:gsub('SUCCESS', '✅ SUCCESS') end",
    },
    
    -- Custom widgets - add live information to your UI
    custom_widgets = {
        -- Git branch indicator
        [[function()
            local handle = io.popen("git branch --show-current 2>/dev/null")
            local branch = handle:read("*a"):gsub("%s+", "")
            handle:close()
            return branch ~= "" and "  " .. branch or ""
        end]],
    }
}
]]

-- ============================================================================
-- THEME PRESETS
-- ============================================================================
-- Uncomment one of these to use a pre-configured theme

-- Dracula Theme
--[[
config.theme = {
    name = "dracula",
    foreground = "#F8F8F2",
    background = "#282A36",
    cursor = "#F8F8F0",
    selection = "#44475A",
    colors = {
        black = "#21222C",
        red = "#FF5555",
        green = "#50FA7B",
        yellow = "#F1FA8C",
        blue = "#BD93F9",
        magenta = "#FF79C6",
        cyan = "#8BE9FD",
        white = "#F8F8F2",
        bright_black = "#6272A4",
        bright_red = "#FF6E6E",
        bright_green = "#69FF94",
        bright_yellow = "#FFFFA5",
        bright_blue = "#D6ACFF",
        bright_magenta = "#FF92DF",
        bright_cyan = "#A4FFFF",
        bright_white = "#FFFFFF"
    }
}
]]

-- Nord Theme
--[[
config.theme = {
    name = "nord",
    foreground = "#D8DEE9",
    background = "#2E3440",
    cursor = "#88C0D0",
    selection = "#4C566A",
    colors = {
        black = "#3B4252",
        red = "#BF616A",
        green = "#A3BE8C",
        yellow = "#EBCB8B",
        blue = "#81A1C1",
        magenta = "#B48EAD",
        cyan = "#88C0D0",
        white = "#E5E9F0",
        bright_black = "#4C566A",
        bright_red = "#D08770",
        bright_green = "#8FBCBB",
        bright_yellow = "#ECEFF4",
        bright_blue = "#5E81AC",
        bright_magenta = "#B48EAD",
        bright_cyan = "#8FBCBB",
        bright_white = "#ECEFF4"
    }
}
]]

-- Solarized Dark
--[[
config.theme = {
    name = "solarized-dark",
    foreground = "#839496",
    background = "#002B36",
    cursor = "#93A1A1",
    selection = "#073642",
    colors = {
        black = "#073642",
        red = "#DC322F",
        green = "#859900",
        yellow = "#B58900",
        blue = "#268BD2",
        magenta = "#D33682",
        cyan = "#2AA198",
        white = "#EEE8D5",
        bright_black = "#002B36",
        bright_red = "#CB4B16",
        bright_green = "#586E75",
        bright_yellow = "#657B83",
        bright_blue = "#839496",
        bright_magenta = "#6C71C4",
        bright_cyan = "#93A1A1",
        bright_white = "#FDF6E3"
    }
}
]]

-- ============================================================================
-- PLATFORM-SPECIFIC CONFIGURATIONS
-- ============================================================================
-- Automatically adjust settings based on operating system

-- Detect operating system
local is_windows = package.config:sub(1,1) == "\\"
local is_unix = not is_windows

if is_windows then
    -- Windows-specific settings
    if not config.shell.default_shell then
        -- Try PowerShell Core first, fallback to Windows PowerShell
        local pwsh_exists = os.execute("where pwsh.exe >nul 2>&1")
        if pwsh_exists == 0 then
            config.shell.default_shell = "pwsh.exe"
        else
            config.shell.default_shell = "powershell.exe"
        end
    end
elseif is_unix then
    -- Unix-specific settings (Linux/macOS)
    if not config.shell.default_shell then
        -- Use $SHELL environment variable
        config.shell.default_shell = os.getenv("SHELL") or "/bin/bash"
    end
end

-- ============================================================================
-- PERFORMANCE TUNING
-- ============================================================================
-- Adjust these settings based on your system capabilities

-- Detect available memory and adjust scrollback accordingly
-- This is a simple heuristic - adjust as needed
local function get_optimal_scrollback()
    -- On most modern systems, 10000 lines is a good default
    -- Adjust if you have limited memory or need more history
    return 10000
end

-- Only override if not already set by user
if config.terminal.scrollback_lines == 10000 then
    -- User hasn't changed it, so we can optimize
    config.terminal.scrollback_lines = get_optimal_scrollback()
end

-- ============================================================================
-- DYNAMIC THEME SWITCHING (Optional)
-- ============================================================================
-- Automatically switch theme based on time of day
-- Uncomment to enable:
--[[
local function get_theme_by_time()
    local hour = tonumber(os.date("%H"))
    
    if hour >= 6 and hour < 18 then
        -- Daytime: Light theme
        return {
            name = "furnace-light",
            foreground = "#2E3440",
            background = "#ECEFF4",
            cursor = "#5E81AC",
            selection = "#D8DEE9",
            colors = {
                black = "#3B4252",
                red = "#BF616A",
                green = "#A3BE8C",
                yellow = "#EBCB8B",
                blue = "#81A1C1",
                magenta = "#B48EAD",
                cyan = "#88C0D0",
                white = "#E5E9F0",
                bright_black = "#4C566A",
                bright_red = "#D08770",
                bright_green = "#8FBCBB",
                bright_yellow = "#ECEFF4",
                bright_blue = "#5E81AC",
                bright_magenta = "#B48EAD",
                bright_cyan = "#8FBCBB",
                bright_white = "#ECEFF4"
            }
        }
    else
        -- Nighttime: Dark theme (use current theme)
        return config.theme
    end
end

-- Apply dynamic theme
config.theme = get_theme_by_time()
]]

-- ============================================================================
-- VALIDATION AND FINAL NOTES
-- ============================================================================
-- The config table must be defined at global scope for Furnace to load it

-- You can add validation here to ensure your configuration is valid
-- Example:
assert(config.terminal.scrollback_lines > 0, "scrollback_lines must be positive")
assert(config.terminal.font_size > 0, "font_size must be positive")

-- Debug: Print confirmation (comment out for production)
-- print("Furnace configuration loaded successfully")
-- print("Theme: " .. config.theme.name)
-- print("Tabs enabled: " .. tostring(config.terminal.enable_tabs))

-- ============================================================================
-- END OF CONFIGURATION
-- ============================================================================
-- For more examples and advanced configurations, see:
-- - config.example.lua (advanced examples with all features)
-- - https://github.com/RyAnPr1Me/furnace/wiki/Configuration
-- ============================================================================
