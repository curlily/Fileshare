# Fileshare

A self-hosted file sharing server written in Rust. Serves a directory over HTTP with a built-in web UI and per-file access control: password protection, token-based links, and hidden files all managed from the command line.

> **Note:** It is strongly recommended to put Fileshare behind a reverse proxy (nginx, Caddy, etc.) to enable HTTPS/TLS before exposing it to a network.

## Features

- **Web UI** - embedded frontend served directly from the binary, no separate deployment needed
- **Directory browsing** - serves any configured directory; folders and files are listed and navigable
- **Password protection** - per-file Argon2-hashed passwords; password is passed as a query parameter (`?password=...`)
- **Token-based access** - generate shareable tokens per file with optional expiry (`1d`, `6h`, `30m`); required to access hidden files
- **Hidden files** - files can be hidden from directory listings and made token-only accessible
- **Live meta reload** - `Meta.toml` is watched for changes and reloaded without restarting the server
- **Path traversal protection** - all resolved paths are validated against the configured base directory

## Installation

```bash
git clone https://github.com/curlily/Fileshare
cd Fileshare
cargo build --release
```

The binary will be at `target/release/fileshare`.

## Configuration

On first run, `Config.toml` is created automatically. Edit it after initial start:

```toml
# Directory to serve
base_directory = '/path/to/your/files'

[server]
host = "localhost"
port = 8080
```

Set `host` to `0.0.0.0` to listen on all interfaces (do this behind a reverse proxy only).

## Usage

### Start / Stop

```bash
fileshare start
fileshare stop
```

`start` writes a PID file (`fileshare.pid`) in the working directory. `stop` reads it and terminates the process.

### File metadata

All metadata is stored in `Meta.toml` and managed via the `meta` subcommand.

#### Tokens

```bash
# Generate a permanent token
fileshare meta add-token path/to/file.zip

# Generate a token that expires in 24 hours
fileshare meta add-token path/to/file.zip --expires 24h

# Other expiry formats: 7d (days), 30m (minutes)
fileshare meta add-token path/to/file.zip --expires 7d

# Remove a token
fileshare meta remove-token path/to/file.zip <token_value>
```

`add-token` prints the token and a ready-to-use URL:

```
Token for file.zip created: abc123...
URL: http://localhost:8080/api/files/path/to/file.zip?token=abc123...
```

#### Password protection

```bash
# Set or update a password
fileshare meta set-password path/to/file.zip xxxxxxxx

# Remove password protection
fileshare meta clear-password path/to/file.zip
```

Password-protected files require `?password=<password>` in the request URL.

#### Hiding files

```bash
# Hide a file from directory listings
fileshare meta hide path/to/file.zip

# Unhide it
fileshare meta unhide path/to/file.zip
```

Hidden files don't appear in listings and require a valid token to access directly.

#### Inspect metadata

```bash
fileshare meta list path/to/file.zip
```

### API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/files` | List root directory |
| `GET` | `/api/files/<path>` | List subdirectory or download file |
| `HEAD` | `/api/files/<path>` | Check file existence/headers |

Query parameters: `?token=<token>`, `?password=<password>`
