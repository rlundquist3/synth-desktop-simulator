# Synth Desktop Simulator

This repo is a wrapper for the core logic in https://github.com/rlundquist3/hardware-synth for prototyping and iteration on a development machine. Its structure (written with Tokio) is set up to closely mirror that of the hardware project (Embassy), rather than necessarily being the optimal setup for a desktop app.

That is, this is not intended to be a usable instrument, rather a playground for use in development of the DSP code in https://github.com/rlundquist3/hardware-synth/tree/main/synth-core. Take a look at that repo for more info on the actual synth code. This is just a dev tool.

### Running

Clone https://github.com/rlundquist3/hardware-synth adjacent to this repo (or edit the `synth-core` dependency in Cargo.toml accordingly if cloned elsewhere).

Requires [Rust & Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) and SDL2 (see below)

---

#### SDL2

**Copied from the [Embedded Graphics Simulator docs](https://docs.rs/embedded-graphics-simulator/latest/embedded_graphics_simulator/):**
The simulator uses SDL2 and its development libraries which must be installed to build and run it.
Linux (apt)

```bash
sudo apt install libsdl2-dev
```

macOS (brew)

```bash
brew install sdl2
```

Users on Apple silicon or with custom installation directories will need to set LIBRARY_PATH for the linker to find the installed SDL2 package:

```bash
export LIBRARY_PATH="$LIBRARY_PATH:$(brew --prefix)/lib"
```

More information can be found in the [SDL2 documentation](https://github.com/Rust-SDL2/rust-sdl2#homebrew).

---

#### MIDI

A MIDI keyboard is required to play. At this time, MIDI may be used for note events, but not control change events (i.e. you can use MIDI to play, but not yet to change parameters). If you have a MIDI controller, plug it in prior to startup, and it should be automatically found.

---

Once those requirements are met:

```bash
cargo build
cargo run
```

- the Embedded Graphics Simulator will open in a new window
- if you have a MIDI controller connected it should connect automatically
- you should see logs similar to the following and be able to start playing

```
Attempting to open MIDI connection
Display initialized
MIDI input port found: MPK mini IV MIDI Port
MIDI connection open, reading input from 'MPK mini IV MIDI Port'...
```

- use a MIDI keyboard to play notes
- The hardware instrument uses a D-pad for navigation and 4 rotary encoders for updating parameters. Here, the computer keyboard is used for navigation, with the equivalent controls:
  | key | hardware equivalent |
  | ------------- | -------------------------- |
  | **↑/↓/←/→** | navigation |
  | **Enter** | select |
  | **1** | encoder 1 clockwise |
  | **Shift + 1** | encoder 1 counterclockwise |
  | **2** | encoder 2 clockwise |
  | **Shift + 2** | encoder 2 counterclockwise |
  | **3** | encoder 3 clockwise |
  | **Shift + 3** | encoder 3 counterclockwise |
  | **4** | encoder 4 clockwise |
  | **Shift + 4** | encoder 4 counterclockwise |
