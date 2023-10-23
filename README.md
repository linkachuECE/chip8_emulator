# chip8_emulator

An emulator for running CHIP-8 games using Rust and SDL2

To run, you must first download the Rust package manager Cargo:
```
curl https://sh.rustup.rs -sSf | sh
```

Then download SDL2:
```
sudo apt-get install libsdl2-dev
```

Navigate to the root directory and run the emulator using

```
cargo run /path/to/rom
```

You can find a collection of usable CHIP-8 ROMs [here](https://www.zophar.net/pdroms/chip8/chip-8-games-pack.html)

This emulator was made using [An Introduction to Chip-8 Emulation using the Rust Programming Language](https://github.com/aquova/chip8-book) and [Cowgod's Chip-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#keyboard)
