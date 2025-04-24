# bm

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
  cd \"$(bm)\"
}
