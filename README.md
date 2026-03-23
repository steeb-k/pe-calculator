# pe-calculator

A basic four-function calculator for Windows PE environments.

## Purpose

Windows PE (Preinstallation Environment) strips out most of the OS — no .NET runtime, no WebView2, no WPF. Standard GUI toolkits that depend on those components won't run. This calculator is built specifically for that context: it has **zero runtime dependencies** beyond what is already present in a minimal WinPE image (`user32.dll`, `gdi32.dll`, `dwmapi.dll` for the optional dark title bar).

## Features

- Addition, subtraction, multiplication, division
- Keyboard input (number row, operators, Enter, Backspace, Escape)
- Dark theme matching the Windows 11 Calculator aesthetic
- Dark title bar (applied automatically if DWM is available)
- Single portable `.exe`, no installer

## Building

Requires Rust and the MSVC toolchain.

```powershell
# Run dev build and launch
.\run-dev.ps1

# Build optimized portable exe → portable\Calculator.exe
.\build-portable.ps1
```

## Compatibility

Tested against Windows 10/11 and Windows PE based on those versions. The binary links only to `kernel32.dll`, `user32.dll`, and `gdi32.dll` — all present in every WinPE image. The dark title bar call (`dwmapi.dll`) is loaded at runtime and silently skipped if unavailable.
