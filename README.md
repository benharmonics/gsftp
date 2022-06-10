# gsftp
SFTP with a text-based graphical interface

Transfer files through an encrypted connection with a visual interface, so you can see both connections at once.

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
- `a`: toggle hidden files
- `q` or `Esc`: quit
- `?`: toggle help menu

## Installation
Note that you will need the development packages of openssl installed.

Ubuntu
```bash
sudo apt install libssl-dev
```
Fedora
```bash
dnf install openssl-devel
```

### Cargo

Clone the repository (i.e. `git clone https://github.com/benharmonics/gsftp.git`), then
```bash
cargo install --path path/to/gsftp
```
Cargo will automatically install programs to `$HOME/.cargo` by default.
