# gsftp
SFTP with a graphical interface

Transfer files through an encrypted connection with a graphical interface, so you can see both connections at once.

Use VIM keys or arrow keys for navigation!

![usage](images/tty.gif)

## Controls

- `l` or `➡` (right arrow key): enter highlighted directory (move further down the directory tree)
- `h` or `⬅` (left arrow key): exit current directory (move further up the directory tree)
- `j` or `⬇` (down arrow key): move down
- `k` or `⬆` (up arrow key): move up
- `y` or `↩` (enter): download/upload highlighted item
- `w` or `↹` (tab): Switch windows
- `b` or `Ctrl`+`⬇`: navigate to bottom-most entry
- `g` or `Ctrl`+`⬆`: navigate to top-most entry
- `q` or `Esc`: quit
- `?`: toggle help menu

## Installation

### Cargo

Clone the repository, then
```bash
cargo install --path path/to/gsftp
```
Cargo will automatically install programs to `$HOME/.cargo` by default.
