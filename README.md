# nginxfmt

A fast, lean command-line tool that formats NGINX configuration files.

This project is not affiliated with or endorsed by NGINX.

## Contents

- [Install](#install)
  - [GitHub Releases](#github-releases-all-platforms)
  - [Homebrew](#homebrew-macos--linux)
  - [Scoop](#scoop-windows)
  - [Winget](#winget-windows)
  - [Debian / Ubuntu](#debian--ubuntu)
  - [Arch Linux (AUR)](#arch-linux-aur)
  - [Chocolatey](#chocolatey-windows)
  - [Cargo](#cargo)
- [Usage](#usage)
- [Configuration](#configuration)
- [Options](#options)
- [Lua blocks](#lua-blocks)
- [License](#license)

## Install

The [GitHub release](https://github.com/nginxfmt/nginxfmt/releases) archives and Linux packages (`.deb`, `.rpm`, `.apk`) are published on every tagged release.

### GitHub Releases (all platforms)

Download the archive for your OS/architecture from the [latest release](https://github.com/nginxfmt/nginxfmt/releases), extract it, and place `nginxfmt` on your `PATH`.

### Homebrew (macOS / Linux)

```bash
brew tap nginxfmt/homebrew-tap
brew install --cask nginxfmt
```

### Scoop (Windows)

```powershell
scoop bucket add nginxfmt https://github.com/nginxfmt/scoop-bucket
scoop install nginxfmt
```

### Winget (Windows)

Manifests are published to [nginxfmt/winget](https://github.com/nginxfmt/winget) on each release. Install from that repository, or wait until a manifest is submitted to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs):

```powershell
winget install nginxfmt.nginxfmt
```

### Debian / Ubuntu

```bash
# Download the .deb for your architecture from the latest release, then:
sudo dpkg -i nginxfmt_*_amd64.deb
```

Also available: `.rpm` (Fedora/RHEL), `.apk` (Alpine), and Arch Linux packages on each release.

### Arch Linux (AUR)

```bash
yay -S nginxfmt-bin
```

### Chocolatey (Windows)

Chocolatey `.nupkg` files are attached to GitHub Releases.

```powershell
choco install nginxfmt
```

### Cargo

Published to [crates.io](https://crates.io/crates/nginxfmt) on each release:

```bash
cargo install nginxfmt
```

## Usage

Format and print to stdout (default):

```bash
nginxfmt conf/nginx.conf
```

Format in place:

```bash
nginxfmt --write conf/nginx.conf
```

Check formatting (useful in CI):

```bash
nginxfmt --check conf/nginx.conf
```

Read from stdin:

```bash
cat conf/nginx.conf | nginxfmt -
```

## Configuration

Create `.nginxfmt.toml` in your project (auto-discovered by walking up from the input file):

```toml
indent_style = "spaces"   # or "tabs"
indent_width = 4
brace_style = "same_line" # or "next_line"
max_blank_lines = 1
trailing_newline = true
preserve_inline_comments = true
```

CLI flags override config file values:

```bash
nginxfmt --tabs --indent-width 2 --brace-style next_line conf/nginx.conf
```

## Options

| Option | Description | Default |
| --- | --- | --- |
| `--write`, `-w` | Write formatted output back to the file | stdout |
| `--check` | Exit 1 if input is not formatted | off |
| `--config PATH` | Explicit config file path | auto-discover |
| `--tabs` / `--spaces` | Indentation style | spaces |
| `--indent-width N` | Spaces per indent level | 4 |
| `--brace-style` | `same_line` or `next_line` | same_line |
| `--max-blank-lines N` | Collapse extra blank lines | 1 |
| `--trailing-newline` | Ensure trailing newline | true |
| `--no-trailing-newline` | Omit trailing newline | off |
| `--preserve-inline-comments` | Keep `#` comments on same line | true |
| `--no-preserve-inline-comments` | Strip inline comments | off |

## Lua blocks

Directives ending in `_by_lua_block` are treated as opaque code blocks. The inner
Lua source is preserved verbatim; only the wrapper indentation is adjusted.

## License

GPL-3.0-or-later. See [LICENSE](LICENSE) for details.
