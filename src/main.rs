use std::collections::BTreeSet;
use std::process::{self, Command};

use clap::{Parser, Subcommand};
use zbus::blocking::{Connection, Proxy};

const BUS_NAME: &str = "org.freedesktop.locale1";
const OBJECT_PATH: &str = "/org/freedesktop/locale1";
const INTERFACE: &str = "org.freedesktop.locale1";

/// Query or change system locale and keyboard settings.
#[derive(Parser)]
#[command(
    name = "localectl",
    override_usage = "localectl [OPTIONS...] COMMAND ..."
)]
struct Cli {
    /// Don't convert keyboard mappings
    #[arg(long, global = true)]
    no_convert: bool,

    /// Do not prompt for password
    #[arg(long, global = true)]
    no_ask_password: bool,

    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show current settings
    Status,

    /// Set system locale
    #[command(name = "set-locale")]
    SetLocale {
        /// LOCALE or VAR=VAL assignments
        #[arg(required = true)]
        locale: Vec<String>,
    },

    /// Show known locales
    #[command(name = "list-locales")]
    ListLocales,

    /// Set virtual console and X11 keyboard mappings
    #[command(name = "set-keymap")]
    SetKeymap {
        /// Keymap name
        map: String,
        /// Toggle keymap
        togglemap: Option<String>,
    },

    /// Show known virtual console keyboard mappings
    #[command(name = "list-keymaps")]
    ListKeymaps,

    /// Set X11 and virtual console keyboard mappings
    #[command(name = "set-x11-keymap")]
    SetX11Keymap {
        /// Keyboard layout
        layout: String,
        /// Keyboard model
        model: Option<String>,
        /// Keyboard variant
        variant: Option<String>,
        /// Keyboard options
        options: Option<String>,
    },

    /// Show known X11 keyboard mapping models
    #[command(name = "list-x11-keymap-models")]
    ListX11KeymapModels,

    /// Show known X11 keyboard mapping layouts
    #[command(name = "list-x11-keymap-layouts")]
    ListX11KeymapLayouts,

    /// Show known X11 keyboard mapping variants
    #[command(name = "list-x11-keymap-variants")]
    ListX11KeymapVariants {
        /// Filter by layout
        layout: Option<String>,
    },

    /// Show known X11 keyboard mapping options
    #[command(name = "list-x11-keymap-options")]
    ListX11KeymapOptions,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        None | Some(Cmd::Status) => cmd_status(),
        Some(Cmd::SetLocale { locale }) => {
            let refs: Vec<&str> = locale.iter().map(|s| s.as_str()).collect();
            cmd_set_locale(&refs)
        }
        Some(Cmd::ListLocales) => cmd_list_locales(),
        Some(Cmd::SetKeymap { map, togglemap }) => {
            cmd_set_keymap(&map, togglemap.as_deref().unwrap_or(""), cli.no_convert)
        }
        Some(Cmd::ListKeymaps) => cmd_list_keymaps(),
        Some(Cmd::SetX11Keymap {
            layout,
            model,
            variant,
            options,
        }) => cmd_set_x11_keymap(
            &layout,
            model.as_deref().unwrap_or(""),
            variant.as_deref().unwrap_or(""),
            options.as_deref().unwrap_or(""),
            cli.no_convert,
        ),
        Some(Cmd::ListX11KeymapModels) => cmd_list_x11("models", None),
        Some(Cmd::ListX11KeymapLayouts) => cmd_list_x11("layouts", None),
        Some(Cmd::ListX11KeymapVariants { layout }) => cmd_list_x11("variants", layout.as_deref()),
        Some(Cmd::ListX11KeymapOptions) => cmd_list_x11("options", None),
    };

    if let Err(e) = result {
        eprintln!("Failed to execute operation: {e}");
        process::exit(1);
    }
}

// ---------------------------------------------------------------------------
// D-Bus helpers (zbus blocking proxy)
// ---------------------------------------------------------------------------

fn locale1_proxy() -> Result<Proxy<'static>, String> {
    let conn = Connection::system().map_err(|e| format!("Failed to connect to system bus: {e}"))?;

    Proxy::new(&conn, BUS_NAME, OBJECT_PATH, INTERFACE)
        .map_err(|e| format!("Failed to create proxy: {e}"))
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

fn cmd_status() -> Result<(), String> {
    let proxy = locale1_proxy()?;

    let locale: Vec<String> = proxy
        .get_property("Locale")
        .map_err(|e| format!("Failed to get Locale: {e}"))?;

    let x11_layout: String = proxy.get_property("X11Layout").map_err(|e| e.to_string())?;
    let x11_model: String = proxy.get_property("X11Model").map_err(|e| e.to_string())?;
    let x11_variant: String = proxy
        .get_property("X11Variant")
        .map_err(|e| e.to_string())?;
    let x11_options: String = proxy
        .get_property("X11Options")
        .map_err(|e| e.to_string())?;
    let vc_keymap: String = proxy
        .get_property("VConsoleKeymap")
        .map_err(|e| e.to_string())?;
    let vc_keymap_toggle: String = proxy
        .get_property("VConsoleKeymapToggle")
        .map_err(|e| e.to_string())?;

    println!(
        "   System Locale: {}",
        if locale.is_empty() {
            "n/a".to_string()
        } else {
            locale[0].clone()
        }
    );
    for entry in locale.iter().skip(1) {
        println!("                  {entry}");
    }

    println!(
        "       VC Keymap: {}",
        if vc_keymap.is_empty() {
            "n/a"
        } else {
            &vc_keymap
        }
    );
    if !vc_keymap_toggle.is_empty() {
        println!("  VC Toggle Keymap: {vc_keymap_toggle}");
    }

    println!(
        "      X11 Layout: {}",
        if x11_layout.is_empty() {
            "n/a"
        } else {
            &x11_layout
        }
    );
    if !x11_model.is_empty() {
        println!("       X11 Model: {x11_model}");
    }
    if !x11_variant.is_empty() {
        println!("     X11 Variant: {x11_variant}");
    }
    if !x11_options.is_empty() {
        println!("     X11 Options: {x11_options}");
    }

    Ok(())
}

fn cmd_set_locale(args: &[&str]) -> Result<(), String> {
    if args.is_empty() {
        return Err("No locale specified.".into());
    }

    let proxy = locale1_proxy()?;
    let locale: Vec<&str> = args.to_vec();
    let interactive = true;

    proxy
        .call_method("SetLocale", &(locale, interactive))
        .map_err(|e| format!("SetLocale: {e}"))?;

    Ok(())
}

fn cmd_list_locales() -> Result<(), String> {
    let output = Command::new("locale")
        .arg("-a")
        .output()
        .map_err(|e| format!("Failed to run 'locale -a': {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("locale -a failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut locales: Vec<&str> = stdout.lines().collect();
    locales.sort();
    for locale in locales {
        println!("{locale}");
    }

    Ok(())
}

fn cmd_set_keymap(keymap: &str, toggle: &str, no_convert: bool) -> Result<(), String> {
    let proxy = locale1_proxy()?;
    let convert = !no_convert;
    let interactive = true;

    proxy
        .call_method(
            "SetVConsoleKeyboard",
            &(keymap, toggle, convert, interactive),
        )
        .map_err(|e| format!("SetVConsoleKeyboard: {e}"))?;

    Ok(())
}

fn cmd_list_keymaps() -> Result<(), String> {
    let mut keymaps = BTreeSet::new();

    let output = Command::new("find")
        .args([
            "/usr/share/kbd/keymaps/",
            "-name",
            "*.map.gz",
            "-o",
            "-name",
            "*.map",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if let Some(name) = std::path::Path::new(line)
                    .file_name()
                    .and_then(|f| f.to_str())
                {
                    let name = name
                        .strip_suffix(".map.gz")
                        .or_else(|| name.strip_suffix(".map"))
                        .unwrap_or(name);
                    keymaps.insert(name.to_string());
                }
            }
        }
    }

    for keymap in &keymaps {
        println!("{keymap}");
    }

    Ok(())
}

fn cmd_set_x11_keymap(
    layout: &str,
    model: &str,
    variant: &str,
    options: &str,
    no_convert: bool,
) -> Result<(), String> {
    let proxy = locale1_proxy()?;
    let convert = !no_convert;
    let interactive = true;

    proxy
        .call_method(
            "SetX11Keyboard",
            &(layout, model, variant, options, convert, interactive),
        )
        .map_err(|e| format!("SetX11Keyboard: {e}"))?;

    Ok(())
}

fn cmd_list_x11(what: &str, layout_filter: Option<&str>) -> Result<(), String> {
    let rules_path = "/usr/share/X11/xkb/rules/base.lst";
    let content = std::fs::read_to_string(rules_path)
        .map_err(|e| format!("Failed to read {rules_path}: {e}"))?;

    let section = match what {
        "models" => "! model",
        "layouts" => "! layout",
        "variants" => "! variant",
        "options" => "! option",
        _ => unreachable!(),
    };

    let mut in_section = false;
    let mut results = Vec::new();

    for line in content.lines() {
        if line.starts_with('!') {
            in_section = line.trim() == section;
            continue;
        }
        if !in_section {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }

        let trimmed = line.trim();

        if what == "variants" {
            // Format: "variant_name  layout: Description"
            let name = trimmed.split_whitespace().next().unwrap_or("");
            if let Some(filter) = layout_filter {
                if let Some(rest) = trimmed.strip_prefix(name) {
                    let rest = rest.trim();
                    if let Some(layout_part) = rest.split(':').next() {
                        if layout_part.trim() != filter {
                            continue;
                        }
                    }
                }
            }
            println!("{name}");
        } else {
            let name = trimmed.split_whitespace().next().unwrap_or("");
            results.push(name.to_string());
        }
    }

    if what != "variants" {
        results.sort();
        for name in &results {
            println!("{name}");
        }
    }

    Ok(())
}
