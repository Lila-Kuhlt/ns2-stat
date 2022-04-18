# ns2-stat-api

A web API for statistics about NS2 games as JSON. To install it, run `cargo install --path ns2-stat-api`.

Available endpoints:

* `GET /games`:

  The raw game data. Can be filtered by a timestamp range.

  Query parameters:

  - `from` (optional): the starting timestamp
  - `to` (optional): the end timestamp

  Response format:

  ```json
  [
      {
          ...
      },
      ...
  ]
  ```

* `GET /stats`:

  The current stats.

  Response format:

  ```json
  {
      "date": timestamp,
      "data": {
          ...
      }
  }
  ```

* `GET /stats/continuous`:

  The continuous stats. Can be filtered by a timestamp range.

  Query parameters:

  - `from` (optional): the starting timestamp
  - `to` (optional): the end timestamp

  Response format:

  ```json
  [
      {
          "date": timestamp,
          "data": {
              ...
          }
      },
      ...
  ]
  ```
