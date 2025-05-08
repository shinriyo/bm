# bm

## Installation

```sh
# Clone the repository
git clone https://github.com/yourusername/bm.git
cd bm

# Install the binary
cargo install --path .
```

## Demo

![demo.gif](demo.gif)

## Features

- `u`: Add current directory to bookmarks
- `j/k`: Move cursor up/down
- `!`: Delete selected bookmark
- `Enter`: Output selected path and exit
- `q`: Quit UI

## Shell Integration

```sh
function bmgo() {
  cd "$(bm)"
}
