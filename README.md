# pixel-forge

Terminal pixel art editor -- 32x24 canvas, 40-color palette, pencil/eraser/fill tools, and undo.

## Features

- 32x24 pixel canvas rendered with half-block characters for double vertical resolution
- 40-color palette (16 ANSI + 24 extended) with `[`/`]` cycling
- Tools: Pencil, Eraser, Flood Fill
- Unlimited undo with `u` or `Ctrl-Z`
- Clear canvas with `c`
- Sidebar shows active tool, current color swatch, cursor coordinates, and palette grid
- Checkerboard transparency background
- Phosphor-green chrome, full-color canvas

## Install

```
cargo build --release
cp target/release/pixel-forge ~/.local/bin/
```

## Usage

```bash
pixel-forge    # launch the editor
```

## Keybindings

| Key | Action |
|-----|--------|
| `h` / `j` / `k` / `l` | Move cursor left / down / up / right |
| Arrow keys | Move cursor |
| `Space` / `Enter` | Apply current tool |
| `p` | Pencil tool |
| `e` | Eraser tool |
| `f` | Flood fill tool |
| `[` / `]` | Previous / next palette color |
| `u` / `Ctrl-Z` | Undo |
| `c` | Clear canvas |
| `q` | Quit |

Built with Rust + ratatui.
