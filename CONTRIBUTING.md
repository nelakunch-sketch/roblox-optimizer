# Contributing to RobloxOptimizer

Thank you for wanting to contribute! Please read this before opening a PR.

---

## 🛡 Hard Rules (non-negotiable)

Any contribution that violates these will be closed immediately:

- ❌ No `WriteProcessMemory` / `ReadProcessMemory`
- ❌ No DLL injection into any process
- ❌ No kernel driver / rootkit techniques
- ❌ No anti-cheat bypass or Roblox ToS violation
- ❌ No obfuscated code

This project must remain **100% anti-cheat safe**.

---

## 🛠 Development Setup

```powershell
# Prerequisites: Rust stable + MSVC toolchain
rustup toolchain install stable
rustup target add x86_64-pc-windows-msvc

# Clone
git clone https://github.com/yourusername/roblox-optimizer
cd roblox-optimizer

# Run in debug mode (still needs Admin for most features)
cargo run

# Release build
cargo build --release

# Lint
cargo clippy -- -D warnings
cargo fmt --all
```

---

## 📐 Code Style

- Run `cargo fmt` before committing
- No `#[allow(unused)]` or `#[allow(dead_code)]` without explanation
- Keep unsafe blocks as small as possible and comment every `unsafe` call
- Every public function should have a doc comment explaining what WinAPI it uses

---

## 🌿 Branch & PR Flow

1. Fork the repo and create a feature branch: `git checkout -b feat/gpu-priority`
2. Make your changes, run `cargo clippy` and `cargo fmt`
3. Open a PR against `main` — fill in the PR template completely
4. CI must pass (build + lint) before merge

---

## 💡 Ideas Welcome

Good first contributions:
- Power plan switching (`SetActivePwrScheme` / `PowerSetActiveScheme`)
- MMCSS profile registration for the Roblox process
- Auto-detect Roblox launch and re-apply optimizations
- GUI wrapper (egui or native Win32)
- Per-optimization restore / undo functionality

---

## 📜 License

By contributing, you agree that your code will be licensed under the [MIT License](LICENSE).
