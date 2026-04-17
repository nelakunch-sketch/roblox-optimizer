## Description

<!-- Explain what this PR does and why. Link any related issues: Fixes #123 -->

## Type of change

- [ ] 🐛 Bug fix
- [ ] ✨ New optimization / feature
- [ ] 🔧 Refactor / cleanup
- [ ] 📝 Docs / README update
- [ ] 🔒 Security fix

## Safety checklist (required for all code changes)

- [ ] This PR does **not** add `WriteProcessMemory` / `ReadProcessMemory` calls
- [ ] This PR does **not** inject DLLs or code into any process
- [ ] This PR does **not** install kernel drivers or hooks
- [ ] This PR is safe to use alongside Roblox's Byfron/Hyperion anti-cheat

## Testing

- [ ] Tested on Windows 10
- [ ] Tested on Windows 11
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt` applied
- [ ] Ran as Administrator and confirmed the relevant optimizations work

## Screenshots / console output (if applicable)

<!-- Paste terminal output showing the optimization working -->
