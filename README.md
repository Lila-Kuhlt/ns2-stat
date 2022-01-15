# Natural Selection 2 Stat reader

## Generating Types from json
Using [Quicktype](https://github.com/quicktype/quicktype) we can generate our structs using:
```shell
quicktype -o types.rs types --density dense --derive-debug -t GameStats --visibility public *.json
```

