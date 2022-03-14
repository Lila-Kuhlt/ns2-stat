# Natural Selection 2 Stat reader

## Usage
Use `cargo run` to run the program. It currently reads all `test_data/*.json` files and prints statistics about these games.

## Generating Types from json
Using [quicktype](https://github.com/quicktype/quicktype) we can generate our structs using:
```shell
quicktype -o types.rs types --density dense --derive-debug -t GameStats --visibility public *.json
```

