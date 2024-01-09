# ns2-stat-cli

A command line tool to print summarized statistics about NS2 games and team suggestions. To install it, run `cargo install --path ns2-stat-cli`.

```
$ ns2-stat-cli --help
Usage: ns2-stat-cli [OPTIONS] [DATA_PATH]

Arguments:
  [DATA_PATH]  The path for the game data [default: test_data]

Options:
      --teams <TEAMS>...         Show team suggestions
      --marine-com <MARINE_COM>
      --alien-com <ALIEN_COM>
  -h, --help                     Print help
```
