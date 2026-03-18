# published

<p align="center">
  <img src="logos/published-logo-1.png" width="256" alt="published logo" />
</p>

Check if an app name is available on the Apple App Store and Google Play.

Queries both stores in parallel and reports which names are taken.

## Install

```bash
brew install bradleydwyer/tap/published
```

Or from source (Rust 1.85+):

```bash
cargo install --git https://github.com/bradleydwyer/published
```

## Usage

Check a name against both stores:

```
$ published Acornify
Acornify:
  2 available, 0 taken, 0 unknown (410ms)
  available: App Store, Google Play
```

Check multiple names:

```
$ published Acornify Spotify
Acornify:
  2 available, 0 taken, 0 unknown (410ms)
  available: App Store, Google Play

Spotify:
  0 available, 2 taken, 0 unknown (385ms)
  taken: App Store, Google Play
```

### Options

```
-s, --stores <IDS>       Comma-separated store IDs (default: all stores)
-a, --all                Check all stores
-j, --json               JSON output
-v, --verbose            Show per-store detail
    --list-stores        Show available stores
```

### Verbose output

```
$ published -v Acornify
Acornify:
  2 available, 0 taken, 0 unknown (410ms)
  [+] App Store            Available    (387ms)
  [+] Google Play          Available    (410ms)
```

### JSON output

```bash
published -j Acornify
```

Returns structured JSON with per-store results, browse URLs, and timing.

### Stores

Two stores are supported:

| ID | Name | Platform |
|----|------|----------|
| `app_store` | App Store | iOS / macOS |
| `google_play` | Google Play | Android |

Filter with `--stores`:

```bash
published --stores app_store MyApp
```

## Claude Code Skill

published includes a [skill](skill/SKILL.md) for Claude Code. Install it with [equip](https://github.com/bradleydwyer/equip):

```bash
equip install bradleydwyer/published
```

This lets Claude Code check app name availability directly when you ask about it.

## License

MIT
