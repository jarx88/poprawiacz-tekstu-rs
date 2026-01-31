# poprawiacz-tekstu-rs

Modern AI-powered text correction tool written in Rust. Sends text to 4 AI providers simultaneously (OpenAI, Anthropic, Gemini, DeepSeek) and lets you choose the best correction.

![Build Status](https://github.com/jarx88/poprawiacz-tekstu-rs/workflows/Build%20Rust%20Binaries/badge.svg)

**Rust rewrite** of [PoprawiaczTekstuPy](https://github.com/jarx88/PoprawiaczTekstuPy) for better performance and native Linux support.

## ✨ Features

- 🦀 **Native Rust** - Fast, memory-safe, cross-platform
- 🎨 **Modern GUI** - egui immediate mode interface with 4-panel layout
- ⚡ **Global Hotkey** - Ctrl+Shift+C automatically captures selected text
- 🔄 **System Tray** - Minimize to tray with show/quit menu
- 🤖 **4 AI Providers** - OpenAI, Anthropic, Gemini, DeepSeek running concurrently
- 📋 **Auto-paste** - Ctrl+V pastes selected correction
- 🌊 **Streaming** - Real-time text streaming from OpenAI
- ⚙️ **Cancellation** - New hotkey cancels previous requests
- 🎨 **Color-coded Panels** - Each AI has unique color (green, orange, blue, purple)
- 💾 **Settings Dialog** - Manage API keys and models via GUI
- 🔍 **Diff Highlighting** - Word-by-word diff with color coding

## 🚀 Installation

### Option 1: Download Binary (Recommended)

1. Go to [Releases](https://github.com/jarx88/poprawiacz-tekstu-rs/releases)
2. Download for your platform:
   - **Linux**: `poprawiacz-tekstu-rs-linux-x86_64.tar.gz`
   - **Windows**: `poprawiacz-tekstu-rs-windows-x86_64.zip`
3. Extract and run:
   ```bash
   # Linux
   tar -xzf poprawiacz-tekstu-rs-linux-x86_64.tar.gz
   ./poprawiacz-tekstu-rs
   
   # Windows
   # Extract zip and run poprawiacz-tekstu-rs.exe
   ```

### Option 2: Build from Source

**Requirements:**
- Rust 1.70+ (`rustup` recommended)
- Linux: GTK3 development libraries
  ```bash
  sudo apt-get install libgtk-3-dev libcairo2-dev libpango1.0-dev libgdk-pixbuf2.0-dev libatk1.0-dev
  ```

**Build:**
```bash
git clone https://github.com/jarx88/poprawiacz-tekstu-rs.git
cd poprawiacz-tekstu-rs
cargo build --release
./target/release/poprawiacz-tekstu-rs
```

## ⚙️ Configuration

### First Run

1. Launch the application
2. Click **Settings** button
3. Enter your API keys:
   - **OpenAI**: `sk-...` from https://platform.openai.com/api-keys
   - **Anthropic**: `sk-ant-...` from https://console.anthropic.com/
   - **Gemini**: `AIza...` from https://aistudio.google.com/app/apikey
   - **DeepSeek**: `sk-...` from https://platform.deepseek.com/api_keys
4. Select models (or use defaults)
5. Click **Save**

### Configuration File

Settings are stored in `~/.config/poprawiacz-tekstu-rs/config.toml` (Linux) or `%APPDATA%\poprawiacz-tekstu-rs\config.toml` (Windows).

Example:
```toml
[api_keys]
openai = "sk-..."
anthropic = "sk-ant-..."
gemini = "AIza..."
deepseek = "sk-..."

[models]
openai = "gpt-4"
anthropic = "claude-3-5-sonnet-20241022"
gemini = "gemini-1.5-pro"
deepseek = "deepseek-chat"
```

## 🎯 Usage

### Workflow

1. **Select text** in any application
2. **Press Ctrl+Shift+C** - app auto-copies text
3. **Watch 4 panels** - each AI processes with spinner animation
4. **Click best result** - panel highlights in green
5. **Press Ctrl+V** - app auto-pastes correction

### AI Panels

- 🟢 **OpenAI** (green #10a37f) - GPT-4 models, streaming support
- 🟠 **Anthropic** (orange #d97706) - Claude models
- 🔵 **Gemini** (blue #4285f4) - Google Gemini
- 🟣 **DeepSeek** (purple #7c3aed) - DeepSeek Chat

### Hotkeys

- **Ctrl+Shift+C** - Capture text and process
- **Ctrl+V** - Paste selected correction
- **Cancel button** - Stop all API calls
- **Minimize to Tray** - Hide window to system tray

## 🔧 Development

### Project Structure

```
poprawiacz-tekstu-rs/
├── src/
│   ├── api/          # API clients (OpenAI, Anthropic, Gemini, DeepSeek)
│   ├── ui/           # GUI components (streaming panel, settings)
│   ├── platform/     # Keyboard simulation (xdotool/Win32)
│   ├── tray/         # System tray integration
│   ├── app.rs        # Main application
│   ├── clipboard.rs  # Clipboard operations
│   ├── config.rs     # TOML configuration
│   ├── diff.rs       # Diff highlighting
│   ├── error.rs      # Error types
│   ├── hotkey.rs     # Global hotkey manager
│   └── main.rs       # Entry point
├── tests/            # Integration tests
└── examples/         # Example programs
```

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Run tests (159 tests: 111 unit + 48 integration)
cargo test

# Run specific test file
cargo test --test integration_config

# Generate documentation
cargo doc --no-deps --open

# Run clippy linter
cargo clippy

# Format code
cargo fmt
```

### Testing

- **Unit tests**: 111 tests across all modules
- **Integration tests**: 48 tests covering config, API, workflow
- **Total coverage**: 159 tests
- See `INTEGRATION_TESTS.md` for detailed test documentation

### CI/CD

GitHub Actions automatically builds on:
- Push to `main`/`master`
- Pull requests
- Version tags (`v*.*.*`)

Artifacts: Linux tarball, Windows zip

## 📦 Technologies

- **GUI**: [egui](https://github.com/emilk/egui) (immediate mode, pure Rust)
- **Async Runtime**: [tokio](https://tokio.rs/)
- **HTTP**: [reqwest](https://github.com/seanmonstar/reqwest) with streaming
- **System Tray**: [tray-icon](https://github.com/tauri-apps/tray-icon)
- **Global Hotkey**: [global-hotkey](https://github.com/tauri-apps/global-hotkey)
- **Clipboard**: [arboard](https://github.com/1Password/arboard)
- **Config**: [serde](https://serde.rs/) + [toml](https://github.com/toml-rs/toml)
- **Diff**: [similar](https://github.com/mitsuhiko/similar)
- **Logging**: [tracing](https://github.com/tokio-rs/tracing)

## 🐛 Troubleshooting

### Linux

**GTK errors**: Install GTK3 development libraries:
```bash
sudo apt-get install libgtk-3-dev
```

**Hotkey not working**: Check if another app is using Ctrl+Shift+C. App tries fallback: Ctrl+Shift+Alt+C

**xdotool not found**: Install for keyboard simulation:
```bash
sudo apt-get install xdotool
```

### Windows

**Hotkey conflicts**: Some apps (screenshot tools) may block Ctrl+Shift+C. Try fallback or disable conflicting apps.

**SmartScreen warning**: Click "More info" → "Run anyway". App is safe, just unsigned.

### General

**API errors**: Verify API keys in Settings. Check internet connection.

**Performance**: Release builds (`cargo build --release`) are 10-100x faster than debug builds.

## 📝 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

MIT License - see [LICENSE](LICENSE) file

## 🙏 Acknowledgments

- Original Python version: [PoprawiaczTekstuPy](https://github.com/jarx88/PoprawiaczTekstuPy)
- Built with amazing Rust ecosystem libraries
- AI providers: OpenAI, Anthropic, Google, DeepSeek

## ❓ FAQ

**Q: Why Rust instead of Python?**  
A: Better performance, lower memory usage, native cross-platform support, type safety.

**Q: Can I use without API keys?**  
A: No, API keys are required for AI providers.

**Q: Does it work offline?**  
A: No, requires internet for AI API calls.

**Q: Can I add more AI providers?**  
A: Currently hardcoded to 4 providers. Open an issue for feature request.

**Q: macOS support?**  
A: Not tested. Should work with `cargo build`, but tray/hotkey may need adjustments.

---

**Built with ❤️ and 🦀 Rust**
