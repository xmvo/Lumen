# Lumen launcher

A small Tauri window that starts the layer for people who do not use a terminal.

It sits in one folder with:

- `bin\lumen.exe`: the layer (this repository's `victory` binary, renamed)
- `shaders\` (optional): shader containers of your own copy; if present they are translated once per build before the game starts
- `art\` (optional): pictures for the backdrop
- `CONTROLS.txt` (optional): opened by the Controls button

Play checks memory, prepares shaders when there are any, then runs `lumen.exe run <game file> --execute` with the NVIDIA build's switches. The game file is picked under Settings; no game files ship with the launcher. Closing the launcher never closes a running game.

Build (Rust and the MSVC tools; WebView2 is part of Windows 11):

```
cd launcher\src-tauri
cargo build --release
```

The result is `target\release\lumen-launcher.exe`.
