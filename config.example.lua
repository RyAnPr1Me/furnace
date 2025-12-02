-- Furnace Terminal Emulator Configuration
-- This is a Lua configuration file for extremely customizable terminal settings
-- Place this file at: ~/.furnace/config.lua

-- Main configuration table
config = {
    -- Shell configuration
    shell = {
        -- Default shell to use (auto-detected if not specified)
        -- Windows: "pwsh.exe", "powershell.exe", or "cmd.exe"
        -- Linux/Mac: uses $SHELL environment variable
        default_shell = "powershell.exe",
        
        -- Working directory (nil defaults to home directory)
        working_dir = nil,
        
        -- Environment variables to set
        env = {
            -- CUSTOM_VAR = "value",
            -- PATH = "/custom/path:" .. os.getenv("PATH")
        }
    },

    -- Terminal behavior settings
    terminal = {
        -- Maximum number of commands in history
        -- Uses a circular buffer for memory efficiency
        max_history = 10000,
        
        -- Enable multiple tabs (disabled by default)
        -- Set to true to enable tab support
        enable_tabs = false,
        
        -- Enable split panes (disabled by default)
        -- Set to true to enable horizontal and vertical splits
        enable_split_pane = false,
        
        -- Font size in points
        font_size = 12,
        
        -- Cursor style: "block", "underline", or "bar"
        cursor_style = "block",
        
        -- Number of lines to keep in scrollback buffer
        -- Higher values use more memory but allow more history
        scrollback_lines = 10000,
        
        -- Enable hardware acceleration for rendering (recommended)
        -- This enables GPU-accelerated rendering at 170 FPS
        hardware_acceleration = true
    },

    -- Theme and color settings
    theme = {
        name = "default",
        foreground = "#FFFFFF",
        background = "#1E1E1E",
        cursor = "#00FF00",
        selection = "#264F78",
        
        -- ANSI color palette with full 24-bit true color support
        colors = {
            -- Normal colors
            black = "#000000",
            red = "#FF0000",
            green = "#00FF00",
            yellow = "#FFFF00",
            blue = "#0000FF",
            magenta = "#FF00FF",
            cyan = "#00FFFF",
            white = "#FFFFFF",
            
            -- Bright colors
            bright_black = "#808080",
            bright_red = "#FF8080",
            bright_green = "#80FF80",
            bright_yellow = "#FFFF80",
            bright_blue = "#8080FF",
            bright_magenta = "#FF80FF",
            bright_cyan = "#80FFFF",
            bright_white = "#FFFFFF"
        }
    },

    -- Keyboard shortcuts
    -- You can customize these to match your workflow
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
        clear = "Ctrl+L"
    }
}

-- Advanced Lua scripting examples:
-- You can use Lua's power to compute values dynamically

-- Example 1: Conditional configuration based on OS
-- if package.config:sub(1,1) == "\\" then
--     -- Windows
--     config.shell.default_shell = "pwsh.exe"
-- else
--     -- Unix-like
--     config.shell.default_shell = os.getenv("SHELL") or "/bin/bash"
-- end

-- Example 2: Calculate scrollback based on available memory
-- local function get_optimal_scrollback()
--     -- This is just an example; you'd need system info
--     return 10000  -- Default value
-- end
-- config.terminal.scrollback_lines = get_optimal_scrollback()

-- Example 3: Theme switching based on time of day
-- local function get_theme_by_time()
--     local hour = tonumber(os.date("%H"))
--     if hour >= 6 and hour < 18 then
--         return {
--             name = "light",
--             background = "#FFFFFF",
--             foreground = "#000000"
--         }
--     else
--         return {
--             name = "dark",
--             background = "#1E1E1E",
--             foreground = "#FFFFFF"
--         }
--     end
-- end
-- You can uncomment this to enable automatic theme switching:
-- config.theme = get_theme_by_time()

-- Example 4: Custom color scheme generator
-- local function generate_gradient_colors(base_hue)
--     -- Generate a color scheme based on a base hue
--     -- This is a simplified example
--     return {
--         black = "#000000",
--         red = string.format("#%02X0000", math.floor(255 * (base_hue / 360))),
--         -- ... more colors
--     }
-- end

-- Example 5: Environment-specific configuration
-- local env = os.getenv("FURNACE_ENV") or "default"
-- if env == "work" then
--     config.terminal.enable_tabs = true
--     config.terminal.scrollback_lines = 50000
-- elseif env == "minimal" then
--     config.terminal.scrollback_lines = 1000
--     config.terminal.hardware_acceleration = false
-- end

-- Note: The config table MUST be defined at the global scope
-- for Furnace to load it properly
