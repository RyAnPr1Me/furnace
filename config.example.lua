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
        },

        -- Optional: Background image configuration
        -- Uncomment to enable a custom background
        -- background_image = {
        --     image_path = "~/.furnace/backgrounds/wallpaper.png",  -- Path to your image
        --     opacity = 0.3,              -- 0.0 (transparent) to 1.0 (opaque)
        --     mode = "fill",              -- "fill", "fit", "stretch", "tile", "center"
        --     blur = 5.0,                 -- Blur effect strength (0.0 = no blur)
        --     color = "#1E1E1E"           -- Fallback solid color
        -- },

        -- Optional: Cursor trail effect for visual feedback
        -- Uncomment to enable smooth cursor trails
        -- cursor_trail = {
        --     enabled = true,
        --     length = 15,                -- Number of trail positions (higher = longer trail)
        --     color = "#00FF0080",        -- Trail color with alpha (#RRGGBBAA format)
        --     fade_mode = "exponential",  -- "linear", "exponential", "smooth"
        --     width = 1.0,                -- Trail width multiplier
        --     animation_speed = 16        -- Milliseconds per frame (~60 FPS)
        -- }
    },

    -- Optional UI Features (all disabled by default for minimal resource usage)
    -- Uncomment and set to true to enable specific features
    features = {
        command_palette = false,     -- Enable command palette (Ctrl+P)
        resource_monitor = false,    -- Enable resource monitor (Ctrl+R) - shows CPU/memory
        autocomplete = false,        -- Enable command autocomplete
        progress_bar = false,        -- Enable progress bar for running commands
        session_manager = false,     -- Enable session save/restore
        theme_manager = false        -- Enable theme switching
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
    },

    -- Advanced Extensibility: Runtime Hooks
    -- This makes Furnace more extensible than any other terminal emulator
    hooks = {
        -- Execute Lua script on startup
        -- on_startup = "~/.furnace/scripts/startup.lua",
        
        -- Execute Lua script on shutdown
        -- on_shutdown = "~/.furnace/scripts/cleanup.lua",
        
        -- Hook for every key press (receives key event data)
        -- on_key_press = "~/.furnace/scripts/key_logger.lua",
        
        -- Hook when a command starts (receives command string)
        -- on_command_start = "~/.furnace/scripts/track_command.lua",
        
        -- Hook when a command completes (receives exit code)
        -- on_command_end = "~/.furnace/scripts/command_stats.lua",
        
        -- Hook for output processing (receives output text)
        -- on_output = "~/.furnace/scripts/highlight_errors.lua",
        
        -- Hook for terminal bell events
        -- on_bell = "~/.furnace/scripts/notify_bell.lua",
        
        -- Hook for title changes
        -- on_title_change = "~/.furnace/scripts/log_title.lua",
        
        -- Custom keybindings with Lua functions
        -- Bind keys to arbitrary Lua code for ultimate flexibility
        custom_keybindings = {
            -- ["Ctrl+Shift+G"] = "function() print('Custom Git shortcut!') end",
            -- ["Ctrl+Shift+D"] = "function() os.execute('docker ps') end",
        },
        
        -- Output filters - transform output before display
        -- Each filter receives text and returns modified text
        output_filters = {
            -- "function(text) return text:gsub('ERROR', '🔴 ERROR') end",
            -- "function(text) return text:gsub('SUCCESS', '✅ SUCCESS') end",
        },
        
        -- Custom widgets - inject custom UI elements
        -- Each widget is Lua code that renders additional UI
        custom_widgets = {
            -- "function() return 'Git: ' .. io.popen('git branch --show-current'):read() end",
            -- "function() return 'Docker: ' .. io.popen('docker ps -q | wc -l'):read() .. ' containers' end",
        }
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

-- Example 6: Dynamic background based on battery status or time
-- local function get_dynamic_background()
--     local hour = tonumber(os.date("%H"))
--     
--     if hour >= 22 or hour < 6 then
--         -- Night mode: dark with subtle background
--         return {
--             image_path = "~/.furnace/backgrounds/night.png",
--             opacity = 0.15,
--             blur = 8.0,
--             mode = "fill"
--         }
--     elseif hour >= 6 and hour < 12 then
--         -- Morning: lighter background
--         return {
--             color = "#F0F0F0",
--             opacity = 0.05,
--             blur = 0.0
--         }
--     else
--         -- Afternoon/Evening: no background
--         return nil
--     end
-- end
-- config.theme.background_image = get_dynamic_background()

-- Example 7: Animated cursor trail based on performance mode
-- local function get_cursor_trail_config()
--     local perf_mode = os.getenv("FURNACE_PERF") or "normal"
--     
--     if perf_mode == "high" then
--         -- Smooth, long trail for high-performance systems
--         return {
--             enabled = true,
--             length = 20,
--             color = "#00FFFF60",  -- Cyan with transparency
--             fade_mode = "smooth",
--             width = 1.2,
--             animation_speed = 8   -- ~120 FPS
--         }
--     elseif perf_mode == "low" then
--         -- Minimal trail for low-end systems
--         return {
--             enabled = true,
--             length = 5,
--             color = "#FFFFFF40",
--             fade_mode = "linear",
--             width = 0.8,
--             animation_speed = 32  -- ~30 FPS
--         }
--     else
--         -- Standard trail
--         return {
--             enabled = true,
--             length = 10,
--             color = "#00FF0080",
--             fade_mode = "exponential",
--             width = 1.0,
--             animation_speed = 16  -- ~60 FPS
--         }
--     end
-- end
-- config.theme.cursor_trail = get_cursor_trail_config()

-- Example 8: Custom background rotation
-- local backgrounds = {
--     "~/.furnace/backgrounds/bg1.png",
--     "~/.furnace/backgrounds/bg2.png",
--     "~/.furnace/backgrounds/bg3.png"
-- }
-- local function rotate_background()
--     local day_of_week = tonumber(os.date("%w"))  -- 0-6
--     local index = (day_of_week % #backgrounds) + 1
--     return {
--         image_path = backgrounds[index],
--         opacity = 0.25,
--         mode = "fit",
--         blur = 3.0
--     }
-- end
-- config.theme.background_image = rotate_background()

-- Example 9: Gradient background using math
-- local function create_gradient_background()
--     -- This is conceptual - actual gradient would need RGB calculation
--     local minute = tonumber(os.date("%M"))
--     local opacity = 0.1 + (minute / 60) * 0.3  -- Varies from 0.1 to 0.4
--     
--     return {
--         color = "#1E1E2E",  -- Base color
--         opacity = opacity,
--         blur = 0.0
--     }
-- end
-- config.theme.background_image = create_gradient_background()

-- ==========================================
-- ADVANCED EXTENSIBILITY EXAMPLES
-- These demonstrate why Furnace is more extensible than any other terminal
-- ==========================================

-- Example 10: Custom keybindings with Lua functions
-- Unlike other terminals, you can bind keys to arbitrary Lua code
-- config.hooks.custom_keybindings = {
--     ["Ctrl+Shift+G"] = [[
--         function()
--             local branch = io.popen("git branch --show-current 2>/dev/null"):read()
--             if branch then
--                 print("On branch: " .. branch)
--             else
--                 print("Not a git repository")
--             end
--         end
--     ]],
--     ["Ctrl+Shift+D"] = [[
--         function()
--             local count = io.popen("docker ps -q | wc -l"):read()
--             print("Running containers: " .. count)
--         end
--     ]],
-- }

-- Example 11: Output filters for real-time text transformation
-- Automatically highlight errors, warnings, and success messages
-- config.hooks.output_filters = {
--     -- Highlight errors in red
--     "function(text) return text:gsub('([Ee][Rr][Rr][Oo][Rr])', '\\27[31m%1\\27[0m') end",
--     -- Highlight warnings in yellow
--     "function(text) return text:gsub('([Ww][Aa][Rr][Nn][Ii][Nn][Gg])', '\\27[33m%1\\27[0m') end",
--     -- Add emoji indicators
--     "function(text) return text:gsub('SUCCESS', '✅ SUCCESS'):gsub('FAILED', '❌ FAILED') end",
-- }

-- Example 12: Custom widgets for status bar
-- Show live information in your terminal UI
-- config.hooks.custom_widgets = {
--     -- Git branch widget
--     [[function()
--         local handle = io.popen("git branch --show-current 2>/dev/null")
--         local branch = handle:read("*a"):gsub("%s+", "")
--         handle:close()
--         return branch ~= "" and "  " .. branch or ""
--     end]],
--     -- Docker container count
--     [[function()
--         local handle = io.popen("docker ps -q 2>/dev/null | wc -l")
--         local count = handle:read("*a"):gsub("%s+", "")
--         handle:close()
--         return count ~= "0" and " 🐳 " .. count or ""
--     end]],
--     -- Current time
--     [[function()
--         return " 🕐 " .. os.date("%H:%M")
--     end]],
-- }

-- Example 13: Command lifecycle hooks
-- Track command execution, measure performance, log commands
-- config.hooks.on_command_start = [[
--     -- Save start time and command
--     _furnace_cmd_start = os.time()
--     _furnace_cmd_text = command
-- ]]
-- 
-- config.hooks.on_command_end = [[
--     -- Calculate and display execution time
--     if _furnace_cmd_start then
--         local duration = os.time() - _furnace_cmd_start
--         if duration > 5 then
--             print(string.format("⏱️  Command took %d seconds", duration))
--         end
--         -- Log slow commands
--         if duration > 30 then
--             local log = io.open(os.getenv("HOME") .. "/.furnace/slow_commands.log", "a")
--             log:write(string.format("%s: %s (%ds)\n", os.date(), _furnace_cmd_text, duration))
--             log:close()
--         end
--     end
-- ]]

-- Example 14: Conditional configuration based on project
-- Automatically adjust settings based on current directory
-- local function detect_project_type()
--     if io.open(".git", "r") then
--         return "git"
--     elseif io.open("Cargo.toml", "r") then
--         return "rust"
--     elseif io.open("package.json", "r") then
--         return "node"
--     elseif io.open("go.mod", "r") then
--         return "go"
--     end
--     return "default"
-- end
-- 
-- local project = detect_project_type()
-- if project == "rust" then
--     config.hooks.custom_keybindings["Ctrl+Shift+B"] = "function() os.execute('cargo build') end"
--     config.hooks.custom_keybindings["Ctrl+Shift+T"] = "function() os.execute('cargo test') end"
-- elseif project == "node" then
--     config.hooks.custom_keybindings["Ctrl+Shift+B"] = "function() os.execute('npm run build') end"
--     config.hooks.custom_keybindings["Ctrl+Shift+T"] = "function() os.execute('npm test') end"
-- end

-- Example 15: External API integration
-- Connect to external services for notifications, logging, etc.
-- config.hooks.on_command_end = [[
--     -- Send notification to external service
--     if exit_code ~= 0 then
--         local curl_cmd = string.format(
--             "curl -X POST https://api.example.com/notify -d 'command failed: %s'",
--             command
--         )
--         os.execute(curl_cmd .. " >/dev/null 2>&1 &")
--     end
-- ]]

-- Example 16: AI-powered command suggestions
-- Use local LLM or API to suggest commands based on history
-- config.hooks.on_key_press = [[
--     if key == "Tab" and current_input ~= "" then
--         -- Query AI for command suggestions
--         local suggestion = io.popen(
--             "curl -s http://localhost:11434/api/generate -d '{\"prompt\":\"" .. 
--             current_input .. "\"}'"
--         ):read("*a")
--         -- Display suggestion
--     end
-- ]]

-- Note: The config table MUST be defined at the global scope
-- for Furnace to load it properly

