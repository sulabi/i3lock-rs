# i3lock-rs

A high-performance, X11-native screen lock background utility that captures your current desktop, applies a fast blur, and streams it directly to `i3lock`.

## Features

- Captures your screen using X11 (no external screenshot tools)
- Blurs using high-speed, low-memory resizing via [`fast_image_resize`](https://crates.io/crates/fast_image_resize)
- Supports custom image input
- No temp files — streams directly to `i3lock` through `/dev/stdin`
- Optimized for speed, perfect for lightweight tiling WMs like i3 & dwm

### Install dependencies

#### Debian/Ubuntu

```bash
sudo apt install libx11-dev libx11-xcb-dev i3lock
```

#### Arch Linux
```bash
sudo pacman -S libx11 libx11-xcb i3lock
```

## Building

```bash
git clone https://github.com/yourusername/i3lock-rs.git
cd i3lock-rs
cargo build --release
```

The resulting binary will be located at `target/release/i3lock-rs`.

## Usage

```bash
./i3lock-rs [--image path/to/image.png] [i3lock options...]
```

### Examples

Capture screen, blur, lock:

```bash
./i3lock-rs
```

Use custom image:

```bash
./i3lock-rs --image ~/Pictures/lockscreen.jpg
```

Forward arguments to `i3lock`:

```bash
./i3lock-rs -- --color=000000 --show-failed-attempts
```

## How it Works

* If no `--image` is provided, captures the X11 root window.
* Downscales to 1/4 resolution for efficient blur.
* Applies a fast box blur.
* Upscales to full resolution and sends raw RGB data to `i3lock`.
