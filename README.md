# localectl

A replacement of [systemd's `localectl`](https://www.freedesktop.org/software/systemd/man/latest/localectl.html) for systemd-free Linux systems (Artix, Void, Gentoo with OpenRC, etc.). It provides the same CLI interface and D-Bus behaviour so that software expecting `localectl` works unchanged on init systems other than systemd.

## Why

Some software hardcodes `localectl` as a dependency without providing a fallback, making it effectively unusable on systemd-free systems even when a `org.freedesktop.locale1` D-Bus service is present. This shim fills that gap by providing a compatible `localectl` binary backed by any compliant D-Bus implementation.

For example, COSMIC Desktop's settings app invokes `localectl list-locales` and `localectl set-locale` directly to populate its language selection list. Without a working `localectl`, the list shows up empty on non-systemd systems.

## Commands

```
localectl [OPTIONS...] COMMAND ...

Commands:
  status                                Show current settings (default)
  set-locale LOCALE|VAR=VAL...          Set system locale
  list-locales                          Show known locales
  set-keymap MAP [TOGGLEMAP]            Set virtual console keyboard mapping
  list-keymaps                          Show known virtual console keymaps
  set-x11-keymap LAYOUT [MODEL [VARIANT [OPTIONS]]]
                                        Set X11 keyboard mapping
  list-x11-keymap-models                Show known X11 keyboard models
  list-x11-keymap-layouts               Show known X11 keyboard layouts
  list-x11-keymap-variants [LAYOUT]     Show known X11 keyboard variants
  list-x11-keymap-options               Show known X11 keyboard options

Options:
  --no-convert          Don't convert between keyboard mappings
  --no-ask-password     Do not prompt for password
  -h, --help            Show help
```

## Building

```bash
cargo build --release
```

## License

GPL-2.0+
