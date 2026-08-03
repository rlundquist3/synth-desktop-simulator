# Terminal Synth

Rudimentary frequency modulation synthesizer written in Rust, with a terminal UI.

At this time, this is not intended to be a highly usable instrument, rather a playground for learning audio programming/digital signal processing. It only has a range of about an octave and a half, represented on a computer keyboard.

Eventually, it may become a more usable MIDI instrument or evolve into a hardware project.

### Running

Requires [Rust & Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```bash
cargo build
cargo run
```

This will open the FM Synth application in your terminal.

![UI Screenshot](images/ui_screenshot.png)

On the left, there is a map of (computer) keyboard keys laid out like an instrument keyboard, where "e" is A440, "4" is B-flat, "3" is A-flat, etc. The instrument has 5 voices (i.e. 5 keys may be played at once).

On the right are the instrument and effect parameter controls:

- arrow keys navigate to the desired section
- enter selects a section; escape de-selects
- within a section:
  - left/right arrow keys select parameters
  - up/down arrow keys change parameters

![Parameters](images/parameters.png)
