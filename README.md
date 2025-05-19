# EmmyLua Debug Adapter

EmmyLua Debug Adapter is dap based on [EmmyLuaDebugger](https://github.com/EmmyLua/EmmyLuaDebugger)

## basic usage

insert debug code:

### Windows
```lua
package.cpath = package.cpath .. ";<path to emmy_core>/?.dll"
local dbg = require("emmy_core")
dbg.tcpListen("localhost", 9966)
dbg.waitIDE() -- donot need
dbg.breakHere() -- donot need
```

### Other OS
```lua
package.cpath = package.cpath .. ";<path to emmy_core>/?.dll"
local dbg = require("emmy_core")
dbg.tcpListen("localhost", 9966)
dbg.waitIDE() -- donot need
dbg.breakHere() -- donot need
```

And start your program, waitting for dap connected.

### Dap config
```json
{
    "type": "emmylua_new",
    "request": "launch",
    "name": "EmmyLua New Debug",
    "host": "localhost",
    "port": 9966,
    "sourcePaths": [
        "${workspaceFolder}",
    ],
    "ext": [
        ".lua",
        ".lua.txt",
        ".lua.bytes"
    ],
    "ideConnectDebugger": true
}
```

## editor example

TODO: I don't know how to use it in other editor, please help me to add it.
