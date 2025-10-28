<div align="center">

# 🚀 EmmyLua Debug Adapter

<p align="center">
  <strong>A powerful Debug Adapter Protocol (DAP) implementation for Lua debugging</strong>
</p>

<p align="center">  <img src="https://img.shields.io/badge/language-Rust-orange?style=for-the-badge&logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-blue?style=for-the-badge" alt="Platform">
  <img src="https://img.shields.io/badge/lua-5.1%20%7C%205.2%20%7C%205.3%20%7C%205.4%20%7C%20LuaJIT-purple?style=for-the-badge&logo=lua" alt="Lua">
  <img src="https://img.shields.io/badge/LuaJIT-2.0%20%7C%202.1-red?style=for-the-badge" alt="LuaJIT">
</p>

<p align="center">
  Built on top of the robust <a href="https://github.com/EmmyLua/EmmyLuaDebugger">EmmyLuaDebugger</a> foundation
</p>

---

</div>

## ✨ Features

- 🎯 **Debug Adapter Protocol (DAP)** compatible
- 🔧 **Cross-platform** support (Windows, Linux, macOS)
- 🚀 **Easy integration** with any DAP-compatible editor
- 🐛 **Advanced debugging** capabilities for Lua applications
- 🌟 **LuaJIT support** for high-performance Lua applications
- 📁 **Multiple file extensions** support (.lua, .lua.txt, .lua.bytes)
- 🌐 **TCP connection** for remote debugging

## 🚀 Quick Start

### 📦 Installation

1. Download the latest release from the [releases page](https://github.com/EmmyLua/emmylua_dap/releases)
2. Extract the executable to your desired location
3. Ensure the `emmy_core` library is available in your Lua environment

### 🛠️ Setup Your Lua Application

Add the following debug code to your Lua application:

#### 🖥️ Windows
```lua
-- Add the emmy_core library to your package path
package.cpath = package.cpath .. ";<path_to_emmy_core>/?.dll"

-- Initialize the debugger
local dbg = require("emmy_core")
dbg.tcpListen("localhost", 9966)

-- Optional: Wait for IDE connection
-- dbg.waitIDE()

-- Optional: Set a breakpoint
-- dbg.breakHere()
```

#### 🐧 Linux
```lua
-- Add the emmy_core library to your package path
package.cpath = package.cpath .. ";<path_to_emmy_core>/?.so"

-- Initialize the debugger
local dbg = require("emmy_core")
dbg.tcpListen("localhost", 9966)

-- Optional: Wait for IDE connection
-- dbg.waitIDE()

-- Optional: Set a breakpoint
-- dbg.breakHere()
```

#### 🍎 macOS
```lua
-- Add the emmy_core library to your package path
package.cpath = package.cpath .. ";<path_to_emmy_core>/?.dylib"
-- Initialize the debugger
local dbg = require("emmy_core")
dbg.tcpListen("localhost", 9966)

-- Optional: Wait for IDE connection
-- dbg.waitIDE()

-- Optional: Set a breakpoint
-- dbg.breakHere()
```


### ⚙️ DAP Configuration

Create a launch configuration in your editor:

```json
{
    "type": "emmylua_new",
    "request": "launch",
    "name": "🐛 EmmyLua Debug Session",
    "host": "localhost",
    "port": 9966,
    "sourcePaths": [
        "path/to/your/workspace"
    ],
    "ext": [
        ".lua",
        ".lua.txt",
        ".lua.bytes"
    ],
    "ideConnectDebugger": true
}
```

## 🎮 Usage

1. **Add debug code** to your Lua application (see setup section above)
2. **Start your Lua program** - it will wait for the debugger to connect
3. **Launch the debug session** from your editor using the DAP configuration
4. **Set breakpoints** and start debugging! 🎉

## 🔧 Editor Integration

<details>
<summary><b>VS Code</b></summary>

Currently, the EmmyLua extension does not use this project as its DAP implementation.

</details>

<details>
<summary><b>Neovim</b></summary>

1. Install the `nvim-dap` plugin
2. Configure the DAP adapter in your Neovim config:

```lua
local dap = require('dap')

dap.adapters.emmylua = {
  type = 'executable',
  command = '/path/to/emmylua_dap',
  args = {}
}

dap.configurations.lua = {
  {
    type = 'emmylua',
    request = 'launch',
    name = 'EmmyLua Debug',
    host = 'localhost',
    port = 9966,
    sourcePaths = { 'path/to/your/workspace' }, -- maybe exist some env variable
    ext = { '.lua' },
    ideConnectDebugger = true
  }
}
```

3. Start debugging with `:DapContinue`

</details>

<details>
<summary><b>IntelliJ IDEA</b></summary>

1. Install the **"EmmyLua"** or **"LSP4IJ"** plugin
2. Go to **Run** → **Edit Configurations**
3. Add a new `Debug Adapter Protocol` configuration
4. set command path to `emmylua_dap` executable
5. write working directory
6. debug mode: launch
7. debug parameters:
```json
{
  "type": "emmylua_new",
  "request": "launch",
  "name": " EmmyLua Debug Session",
  "host": "localhost",
  "port": 9966,
  "sourcePaths": [
    "${workspaceFolder}"
  ],
  "ext": [
    ".lua",
    ".lua.txt",
    ".lua.bytes"
  ],
  "ideConnectDebugger": true
}

```


</details>

<details>
<summary><b>Zed Editor</b></summary>

1. Open your project in Zed
2. Create or edit `.zed/debug.json`:

```json
[
  {
    "label": "EmmyLua Debug",
    "adapter": "emmylua_new",
    "type": "emmylua_new",
    "request": "launch",
    "host": "localhost",
    "port": 9966,
    "sourcePaths": ["$ZED_WORKTREE_ROOT"],
    "ext": [".lua", ".lua.txt", ".lua.bytes"],
    "ideConnectDebugger": true
  }
]

```

3. Start debugging from the Debug panel

</details>

<details>
<summary><b>Vim (with vim-dap)</b></summary>

1. Install a DAP plugin like `vimspector` or `vim-dap`
2. Configure `.vimspector.json`:

```json
{
  "configurations": {
    "EmmyLua Debug": {
      "adapter": "emmylua",
      "configuration": {
        "request": "launch",
        "host": "localhost",
        "port": 9966,
        "sourcePaths": ["${workspaceFolder}"],
        "ext": [".lua", ".lua.txt", ".lua.bytes"],
        "ideConnectDebugger": true
      }
    }
  },
  "adapters": {
    "emmylua": {
      "command": ["/path/to/emmylua_dap"]
    }
  }
}
```

3. Start debugging with `:VimspectorLaunch`

</details>

<details>
<summary><b>Emacs (with dap-mode)</b></summary>

1. Install `dap-mode` package
2. Add to your Emacs config:

```elisp
(require 'dap-mode)

(dap-register-debug-template
  "EmmyLua Debug"
  (list :type "emmylua"
        :request "launch"
        :name "EmmyLua Debug Session"
        :host "localhost"
        :port 9966
        :sourcePaths (list (lsp-workspace-root))
        :ext (list ".lua" ".lua.txt" ".lua.bytes")
        :ideConnectDebugger t))
```

3. Start debugging with `M-x dap-debug`

</details>

<details>
<summary><b>Other Editors</b></summary>

Any editor that supports the Debug Adapter Protocol can be used with EmmyLua DAP:

- **Eclipse** (with DAP extensions)
- **Sublime Text** (with DAP plugins)
- **Atom** (with DAP packages)
- **Kate** (KDE Advanced Text Editor with DAP support)

General steps:
1. Find and install a DAP plugin/extension for your editor
2. Configure the adapter executable path
3. Set up the launch configuration with the parameters shown above
4. Connect to your Lua application on port 9966

</details>

## 📋 Configuration Options

| Option | Type | Description | Default |
|--------|------|-------------|---------|
| `host` | string | Debug server host | `"localhost"` |
| `port` | number | Debug server port | `9966` |
| `sourcePaths` | array | Source code directories | `["${workspaceFolder}"]` |
| `ext` | array | Supported file extensions | `[".lua", ".lua.txt", ".lua.bytes"]` |
| `ideConnectDebugger` | boolean | IDE initiates connection | `true` |

## 🤝 Contributing

We welcome contributions! Please feel free to:

- 🐛 Report bugs
- 💡 Suggest features  
- 🔧 Submit pull requests
- 📚 Improve documentation

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [EmmyLuaDebugger](https://github.com/EmmyLua/EmmyLuaDebugger) - The core debugging engine
- [emmy_dap_types](https://github.com/EmmyLuaLs/emmy_dap_types)
- Contributors who help improve this project

