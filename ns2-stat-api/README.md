# ns2-stat-api

A web API for statistics about NS2 games as JSON. To install it, run `cargo install --path ns2-stat-api`.

Available endpoints:

* `GET /games`:

  All games in a summarized form. Can be filtered by a Unix time range.

  Query parameters:

  - `from` (optional): the starting time
  - `to` (optional): the end time

  Response format: `Array<GameSummary>`

* `GET /games/latest`

  The latest game in a summarized form.

  Response format: `GameSummary`

* `GET /stats`:

  The current stats.

  Response format: `NS2Stats`

* `GET /stats/continuous`:

  The continuous stats. Can be filtered by a Unix time range.

  Query parameters:

  - `from` (optional): the starting time
  - `to` (optional): the end time

  Response format: `[number: NS2Stats]`

## TypeScript type definitions

```ts
type Stat<T> = {
    total: T,
    marines: T,
    aliens: T,
}

type User = {
    games: Stat<number>,
    commander: Stat<number>,
    wins: Stat<number>,
    kills: Stat<number>,
    assists: Stat<number>,
    deaths: Stat<number>,
    score: Stat<number>,
    hits: Stat<number>,
    misses: Stat<number>,
}

type Map = {
    total_games: number,
    marine_wins: number,
    alien_wins: number,
}

type NS2Stats = {
    latest_game: number,
    users: Record<string, User>,
    maps: Record<string, Map>,
    total_games: number,
    marine_wins: number,
    alien_wins: number,
}

type PlayerSummary = {
    kills: number,
    assists: number,
    deaths: number,
    score: number,
    hits: number,
    misses: number,
}

type TeamSummary = {
    players: Record<string, PlayerSummary>,
    commander: string | null,
    rt_graph: Array<[number, number]>,
}

type WinningTeam = "None" | "Aliens" | "Marines"

type GameSummary = {
    round_date: number,
    winning_team: WinningTeam,
    round_length: number,
    map_name: string,
    aliens: TeamSummary,
    marines: TeamSummary,
}
```
