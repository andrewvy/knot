knot
---

A Minetest proxy designed to facilitate linking multiple servers together under a powerful API.

## Installation

To get started, clone this repo and run the following command:

`cargo install`

## Running knot

Starting `knot` will create a default `config.toml` in the current working directory.

Knot will then begin to listen to new connections on `0.0.0.0:30001` and forward clients to `127.0.0.1:30000` as the default lobby server.

## Configuration

- Default `config.toml`

```toml
# knot config.toml
host = "0.0.0.0:30001"
player_limit = -1

[servers]
  [servers.lobby]
    address = "127.0.0.1:30000"
```

> `host`

```
Type: String
Description: The IP and Port used for the proxy, defaults to 0.0.0.0:30001 to bind to all addresses.
```

> `player_limit`

```
Type: Integer
Description: How many players are allowed to connect to the proxy at one time. Negative values will be treated as no player-limit.
```

> `servers`

```
Type: Table
Description: Used to define servers to be managed under the proxy. Expects a `servers.lobby` to exist.

Defaults to:

[servers.lobby]
  address = "127.0.0.1:30000"
```

> `server specification`

```
address (String): IP + Port of the Minetest server.
```
