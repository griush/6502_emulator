# C64 Emulator
![C64 README Banner](./res/C64_Emulator_banner.png)
![GitHub repo size](https://img.shields.io/github/repo-size/griush/c64_emulator)
![GitHub Repo stars](https://img.shields.io/github/stars/griush/c64_emulator?color=green)
![GitHub License](https://img.shields.io/github/license/griush/c64_emulator?style=flat&color=green)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/griush/c64_emulator/.github%2Fworkflows%2Frust.yml)

C64 emulator made in Rust.
## State
Tesed on Windows 10. But should work on any platform, there is no system specific code.
> [!WARNING]
> Only the Mos6510 is emulated currently, so this is a work-in-progress.
> And you will only see a blue background.
### Short-term plans
Emulating the Vic-ii and the Sid to make a usable C64 emulator.

## Usage
- Clone the repo with `git clone https://github.com/griush/c64_emulator.git`.
- Run `cargo run` to start the emulator. You can pass an argument (`cargo run <path>`) to load a custom binary. However the C64 ROMs will not be loaded. At least that is how it is working now.
