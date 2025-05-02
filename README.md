# Fenix

Fenix is my final project for highschool at the technological highschool for electrical engineering at Merano.
It is a WIP operating system for the AM335 controller and is paired with a custom designed board inspired by the beaglebone black.

It implements the following features:
- Multitasking
- Paging
- User/Kernel Privilege Differentiation
- Syscalls
- I2C
- GPIO
- Pinmuxxing

## Setup

### Requirements

- MicroSD card

### Nix Setup

As I'm using NixOS as my primary operating system I have included a nix flake, which make setting up the project much easier.
run the following command to enter the devShell
```
nix develop
```

### Non Nix Setup

You will need to have the following packages installed

- rustup
- gcc-arm-none-eabi (might be called differently on your distribution)
- qemu (only for emulation)

After installing these packages install the needed rustup target

```
rustup default stable
rustup target add armv7a-none-eabi
```


For compiling the project run `make`, for flashing the sd card `make flash`. If you have multiple external drives, you might have to change the sdX path to the sd card to the correct path

When using the Beaglebone Black, simply insert the SD card and press the Switch No. 2 (S2) located on the top right of the board
