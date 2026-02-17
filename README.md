# Wallity (Iced)

> **Work In Progress**  
> This project is currently under active development. Features may be incomplete or subject to change.

A desktop wallpaper manager built with Rust and Iced, designed for **Wayland/Hyprland** environments.

- Repository: https://github.com/asce4s/wallity-iced
- Releases: https://github.com/asce4s/wallity-iced/releases
- Issues: https://github.com/asce4s/wallity-iced/issues

## Platform Support

This application is specifically designed for:
- **Wayland** compositors

## Features

- Browse and manage wallpapers
- Virtual scrolling for performance
- Keyboard navigation support
- Thumbnail generation and caching
- Config file support

## Tech Stack

- **Language**: Rust (Edition 2024)
- **UI**: Iced 0.14 (`wgpu` + `image` features)
- **Image Processing**: `image`
- **Parallel Processing**: `rayon`
- **Configuration**: `serde` + `toml` + `once_cell`

## Installation

### From Source

1. Clone the repository:
   ```bash
   git clone https://github.com/asce4s/wallity-iced.git
   cd wallity-iced
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. The built binary will be available at:
   ```bash
   target/release/wallity
   ```

4. (Optional) Copy the binary to your PATH:
   ```bash
   sudo cp target/release/wallity /usr/local/bin/
   ```

## System Requirements

- Linux with Wayland compositor (tested with Hyprland)
- One of the following wallpaper setters:
  - `hyprpaper`
  - `swww`
  - Any other tool that can read from a file path

## First Run

1. Create the config directory:
   ```bash
   mkdir -p ~/.config/wallity
   ```

2. (Optional) Create a config file `~/.config/wallity/wallity.toml` with your settings (see Configuration section below)

3. Ensure you have wallpapers in `~/Pictures/wallpapers` or configure a custom path in the config file

## Development

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (with `cargo`)

### Running the Application

#### Development Mode
```bash
cargo run
```

#### Build for Production
```bash
cargo build --release
```

## Configuration

The application reads configuration from `~/.config/wallity/wallity.toml`. If the file does not exist, default values are used.

### Config File Location
```text
~/.config/wallity/wallity.toml
```

### Configuration Options

Create or edit the config file with the following options:

```toml
# Path to the directory containing wallpapers
# Default: ~/Pictures/wallpapers
wallpaper_path = "~/Pictures/wallpapers"

# Path where the current wallpaper symlink will be created
# This symlink points to the currently selected wallpaper
# Default: ~/.config/wallity/.current_wallpaper
current_wallpaper = "~/.config/wallity/.current_wallpaper"

# Script to execute after setting a wallpaper
# For Hyprland, you might use:
# hyprctl hyprpaper wallpaper "eDP-1,~/.config/wallity/.current_wallpaper"
# Default: "" (empty)
post_script = ""

# Directory where thumbnail cache is stored
# Default: ~/.cache/wallity/thumbnails
cache_path = "~/.cache/wallity/thumbnails"
```

### Example Configuration

#### For Hyprland with hyprpaper
```toml
wallpaper_path = "~/Pictures/wallpapers"
current_wallpaper = "~/.config/wallity/.current_wallpaper"
post_script = "hyprctl hyprpaper wallpaper 'eDP-1,~/.config/wallity/.current_wallpaper'"
cache_path = "~/.cache/wallity/thumbnails"
```

#### For Hyprland with swww
```toml
wallpaper_path = "~/Pictures/wallpapers"
current_wallpaper = "~/.config/wallity/.current_wallpaper"
post_script = "swww img ~/.config/wallity/.current_wallpaper"
cache_path = "~/.cache/wallity/thumbnails"
```

### Notes

- All paths support tilde (`~`) expansion
- The `post_script` is executed after the wallpaper symlink is created
- The config and cache directories are created automatically if needed

## License

This project is open source and available under the [MIT License](LICENSE).
