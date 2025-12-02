# Furnace: The Most Extensible Terminal Emulator

## Why Furnace is More Extensible Than Any Other Terminal

Furnace isn't just configurable - it's **fully programmable**. While other terminal emulators limit you to predefined configuration options or restricted scripting, Furnace gives you complete control through a powerful Lua runtime environment.

## Comparison with Other Popular Terminals

### The Competition

**Alacritty**
- Static YAML configuration
- No scripting capability
- Can only adjust visual settings and keybindings to launch external commands

**Kitty**
- Python configuration (limited to setup-time)
- Remote control API (requires external process)
- Cannot intercept or transform terminal events

**WezTerm**
- Lua configuration (good!)
- Limited to configuration-time execution
- No runtime hooks or event system
- Cannot transform output or inject custom UI

**iTerm2**
- AppleScript/Python scripting
- macOS only
- Limited event exposure
- Cannot transform output in real-time

### What Furnace Offers That Others Don't

## 1. Runtime Hook System

Execute Lua code in response to terminal events:

```lua
hooks = {
    on_startup = "~/.furnace/scripts/init.lua",
    on_shutdown = "~/.furnace/scripts/cleanup.lua",
    on_key_press = "~/.furnace/scripts/key_logger.lua",
    on_command_start = "~/.furnace/scripts/track.lua",
    on_command_end = "~/.furnace/scripts/analyze.lua",
    on_output = "~/.furnace/scripts/process.lua",
    on_bell = "~/.furnace/scripts/notify.lua",
    on_title_change = "~/.furnace/scripts/log_title.lua"
}
```

**No other terminal offers this level of event access.**

## 2. Custom Keybindings with Lua Functions

Unlike other terminals where keybindings can only launch predefined actions or external commands, Furnace lets you bind keys to **arbitrary Lua code**:

```lua
hooks.custom_keybindings = {
    -- Check git status with custom formatting
    ["Ctrl+Shift+G"] = [[
        function()
            local branch = io.popen("git branch --show-current"):read()
            local uncommitted = io.popen("git status --short"):read("*a")
            local unpushed = io.popen("git log origin..HEAD --oneline"):read("*a")
            
            print("📍 Branch: " .. branch)
            if uncommitted ~= "" then
                print("📝 Uncommitted changes:")
                print(uncommitted)
            end
            if unpushed ~= "" then
                print("⬆️  Unpushed commits:")
                print(unpushed)
            end
        end
    ]],
    
    -- Docker management
    ["Ctrl+Shift+D"] = [[
        function()
            local running = io.popen("docker ps -q | wc -l"):read()
            local all = io.popen("docker ps -aq | wc -l"):read()
            print("🐳 Docker: " .. running .. " running, " .. all .. " total")
            
            if tonumber(running) > 0 then
                os.execute("docker ps --format 'table {{.Names}}\t{{.Status}}'")
            end
        end
    ]],
    
    -- Project-specific build
    ["Ctrl+Shift+B"] = [[
        function()
            if io.open("Cargo.toml") then
                os.execute("cargo build")
            elseif io.open("package.json") then
                os.execute("npm run build")
            elseif io.open("Makefile") then
                os.execute("make")
            else
                print("No build system detected")
            end
        end
    ]]
}
```

## 3. Real-Time Output Filtering

Transform terminal output before it's displayed:

```lua
hooks.output_filters = {
    -- Highlight errors in red
    [[function(text)
        return text:gsub('([Ee][Rr][Rr][Oo][Rr])', '\27[31;1m%1\27[0m')
    end]],
    
    -- Highlight warnings in yellow
    [[function(text)
        return text:gsub('([Ww][Aa][Rr][Nn][Ii][Nn][Gg])', '\27[33;1m%1\27[0m')
    end]],
    
    -- Add emoji indicators
    [[function(text)
        text = text:gsub('SUCCESS', '✅ SUCCESS')
        text = text:gsub('FAILED', '❌ FAILED')
        text = text:gsub('PASSED', '✔️  PASSED')
        return text
    end]],
    
    -- Auto-format JSON (if you want)
    [[function(text)
        if text:match('^%s*{') then
            -- Detect JSON and pretty-print
            local formatted = io.popen("jq . 2>/dev/null", "w")
            formatted:write(text)
            formatted:close()
        end
        return text
    end]]
}
```

**This is completely impossible in other terminals.**

## 4. Custom Widgets

Inject live, dynamic UI elements:

```lua
hooks.custom_widgets = {
    -- Current git branch
    [[function()
        local handle = io.popen("git branch --show-current 2>/dev/null")
        local branch = handle:read("*a"):gsub("%s+", "")
        handle:close()
        return branch ~= "" and "  " .. branch or ""
    end]],
    
    -- Docker container count
    [[function()
        local handle = io.popen("docker ps -q 2>/dev/null | wc -l")
        local count = handle:read("*a"):gsub("%s+", "")
        handle:close()
        return count ~= "0" and " 🐳 " .. count or ""
    end]],
    
    -- Kubernetes context
    [[function()
        local handle = io.popen("kubectl config current-context 2>/dev/null")
        local ctx = handle:read("*a"):gsub("%s+", "")
        handle:close()
        return ctx ~= "" and " ☸️  " .. ctx or ""
    end]],
    
    -- Current time
    [[function()
        return " 🕐 " .. os.date("%H:%M:%S")
    end]]
}
```

## 5. Command Lifecycle Tracking

Monitor and analyze every command:

```lua
-- Track command start time
hooks.on_command_start = [[
    _furnace_cmd_start = os.time()
    _furnace_cmd_text = command
    _furnace_cmd_count = (_furnace_cmd_count or 0) + 1
]]

-- Analyze on completion
hooks.on_command_end = [[
    if _furnace_cmd_start then
        local duration = os.time() - _furnace_cmd_start
        
        -- Warn on slow commands
        if duration > 5 then
            print(string.format("⏱️  Command took %d seconds", duration))
        end
        
        -- Log very slow commands
        if duration > 30 then
            local log = io.open(os.getenv("HOME") .. "/.furnace/slow_commands.log", "a")
            if log then
                log:write(string.format("%s: %s (%ds)\n", 
                    os.date("%Y-%m-%d %H:%M:%S"), 
                    _furnace_cmd_text, 
                    duration))
                log:close()
            end
        end
        
        -- Notify on failure
        if exit_code ~= 0 then
            os.execute(string.format(
                "notify-send 'Command Failed' 'Exit code: %d\n%s' 2>/dev/null",
                exit_code,
                _furnace_cmd_text
            ))
        end
        
        -- Update statistics
        _furnace_total_time = (_furnace_total_time or 0) + duration
        
        if _furnace_cmd_count % 10 == 0 then
            local avg = _furnace_total_time / _furnace_cmd_count
            print(string.format("📊 Stats: %d commands, avg %.1fs", 
                _furnace_cmd_count, avg))
        end
    end
]]
```

## 6. External Integration

Connect to any external system:

```lua
-- Slack notifications
hooks.on_command_end = [[
    if exit_code ~= 0 and duration > 10 then
        local webhook = "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
        local payload = string.format(
            '{"text":"Command failed: `%s` (exit %d)"}',
            command:gsub('"', '\\"'),
            exit_code
        )
        os.execute(string.format(
            "curl -X POST -H 'Content-type: application/json' --data '%s' %s 2>/dev/null &",
            payload,
            webhook
        ))
    end
]]

-- Prometheus metrics
hooks.on_command_end = [[
    local metric = string.format(
        "command_duration_seconds{command=\"%s\",exit_code=\"%d\"} %d\n",
        command:match("^%S+") or "unknown",
        exit_code,
        duration
    )
    os.execute(string.format(
        "echo '%s' | curl --data-binary @- http://localhost:9091/metrics/job/furnace 2>/dev/null &",
        metric
    ))
]]

-- AI-powered command suggestions
hooks.on_key_press = [[
    if key == "Ctrl+Space" and current_input ~= "" then
        local response = io.popen(string.format(
            "curl -s http://localhost:11434/api/generate -d '{\"model\":\"codellama\",\"prompt\":\"Suggest a shell command for: %s\"}'",
            current_input
        )):read("*a")
        -- Parse and display suggestion
    end
]]
```

## 7. Conditional Configuration

Adjust everything based on context:

```lua
-- Detect project type and configure accordingly
local function detect_and_configure()
    local cwd = io.popen("pwd"):read()
    
    -- Rust project
    if io.open("Cargo.toml") then
        config.hooks.custom_keybindings["Ctrl+Shift+B"] = "os.execute('cargo build')"
        config.hooks.custom_keybindings["Ctrl+Shift+T"] = "os.execute('cargo test')"
        config.hooks.custom_keybindings["Ctrl+Shift+R"] = "os.execute('cargo run')"
    
    -- Node project
    elseif io.open("package.json") then
        config.hooks.custom_keybindings["Ctrl+Shift+B"] = "os.execute('npm run build')"
        config.hooks.custom_keybindings["Ctrl+Shift+T"] = "os.execute('npm test')"
        config.hooks.custom_keybindings["Ctrl+Shift+R"] = "os.execute('npm start')"
    
    -- Python project
    elseif io.open("requirements.txt") or io.open("setup.py") then
        config.hooks.custom_keybindings["Ctrl+Shift+T"] = "os.execute('pytest')"
        config.hooks.custom_keybindings["Ctrl+Shift+R"] = "os.execute('python main.py')"
    end
    
    -- Adjust theme based on environment
    local env = os.getenv("WORK_ENV")
    if env == "production" then
        config.theme.background = "#3D0000"  -- Red tint for production
    elseif env == "staging" then
        config.theme.background = "#3D3D00"  -- Yellow tint for staging
    end
end

detect_and_configure()
```

## Real-World Use Cases

### 1. DevOps Engineer
```lua
-- Show Kubernetes context and namespace in status bar
hooks.custom_widgets = {
    [[function()
        local ctx = io.popen("kubectl config current-context 2>/dev/null"):read()
        local ns = io.popen("kubectl config view --minify -o jsonpath='{..namespace}' 2>/dev/null"):read()
        return string.format(" ☸️  %s/%s", ctx or "none", ns or "default")
    end]]
}

-- Quick commands
hooks.custom_keybindings = {
    ["Ctrl+Shift+K"] = "os.execute('kubectl get pods')",
    ["Ctrl+Shift+L"] = "os.execute('kubectl logs -f $(kubectl get pods -o name | head -1)')"
}
```

### 2. Software Developer
```lua
-- Auto-detect and run appropriate test command
hooks.custom_keybindings = {
    ["Ctrl+Shift+T"] = [[
        function()
            if io.open("Cargo.toml") then os.execute("cargo test")
            elseif io.open("package.json") then os.execute("npm test")
            elseif io.open("go.mod") then os.execute("go test ./...")
            else print("No test command configured") end
        end
    ]]
}

-- Track test execution time
hooks.on_command_start = [[
    if command:match("test") then
        _test_start = os.time()
    end
]]

hooks.on_command_end = [[
    if _test_start then
        print(string.format("Tests completed in %ds", os.time() - _test_start))
        _test_start = nil
    end
]]
```

### 3. Data Scientist
```lua
-- Show conda/venv environment
hooks.custom_widgets = {
    [[function()
        local env = os.getenv("CONDA_DEFAULT_ENV") or os.getenv("VIRTUAL_ENV")
        return env and " 🐍 " .. env:match("[^/]+$") or ""
    end]]
}

-- Quick Jupyter notebook launch
hooks.custom_keybindings = {
    ["Ctrl+Shift+J"] = "os.execute('jupyter notebook &')"
}
```

## Summary

Furnace provides extensibility that goes far beyond any other terminal emulator:

✅ **Runtime hooks** - Execute code on 8+ different events  
✅ **Custom keybindings with Lua** - Not just launch commands, but arbitrary logic  
✅ **Output filtering** - Transform text in real-time  
✅ **Custom widgets** - Inject dynamic UI elements  
✅ **Full Lua 5.4 runtime** - Complete programming environment  
✅ **External integration** - Connect to ANY system or API  
✅ **Conditional configuration** - Adapt based on context  

**This makes Furnace more extensible than 100% of current terminal emulators.**
