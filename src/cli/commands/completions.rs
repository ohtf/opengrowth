// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! `opengrowth completions`: print, write, or install shell completions.
//!
//! The same interface as `clincalc completions`:
//!
//! - `opengrowth completions <SHELL>` prints the script to stdout
//! - `opengrowth completions <SHELL> --dir DIR` writes it to DIR
//! - `opengrowth completions install [--shell SHELL] [--dir DIR]` installs it
//!   for the current user, detecting the shell from `$SHELL`
//!
//! Printing follows the name the binary was run as. Writing and installing
//! produce scripts for both names, `opengrowth` and `og`, since both are
//! installed.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context as _, anyhow};
use clap::CommandFactory;
use clap_complete::Shell;

use crate::cli::paths;

/// The names the binary is installed under.
const NAMES: [&str; 2] = ["opengrowth", "og"];

/// Arguments for `completions`.
#[derive(Debug, clap::Args)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<CompletionCommand>,

    /// Shell to generate completions for.
    pub shell: Option<Shell>,

    /// Write the scripts to this directory instead of printing to stdout.
    #[arg(long, short = 'd', value_parser = paths::tilde_path, value_hint = clap::ValueHint::DirPath)]
    pub dir: Option<PathBuf>,
}

/// `completions` subcommands.
#[derive(Debug, clap::Subcommand)]
pub enum CompletionCommand {
    /// Install completions for the current user, for both `opengrowth` and `og`.
    Install {
        /// Shell to install for. Read from $SHELL when omitted.
        #[arg(long)]
        shell: Option<Shell>,

        /// Directory to install to. Defaults to the shell's per-user location.
        #[arg(long, short = 'd', value_parser = paths::tilde_path, value_hint = clap::ValueHint::DirPath)]
        dir: Option<PathBuf>,
    },
}

/// Runs `completions`.
pub fn run(args: &Args) -> anyhow::Result<ExitCode> {
    match &args.command {
        Some(CompletionCommand::Install { shell, dir }) => {
            let shell = shell.or_else(detect_shell).ok_or_else(|| {
                anyhow!("could not detect your shell from $SHELL; pass --shell bash|zsh|fish|powershell|elvish")
            })?;
            let dir = match dir {
                Some(dir) => dir.clone(),
                None => default_dir(shell)?,
            };
            write_all(shell, &dir)?;
            print_install_note(shell, &dir);
        }
        None => {
            let shell = args.shell.ok_or_else(|| {
                anyhow!("missing shell; for example `opengrowth completions zsh`, or `opengrowth completions install`")
            })?;
            match &args.dir {
                Some(dir) => write_all(shell, dir)?,
                None => {
                    let mut command = crate::cli::Cli::command();
                    clap_complete::generate(
                        shell,
                        &mut command,
                        invoked_name(),
                        &mut std::io::stdout(),
                    );
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn invoked_name() -> &'static str {
    let invoked = std::env::args_os().next().and_then(|arg| {
        Path::new(&arg)
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
    });
    if invoked.as_deref() == Some("og") {
        "og"
    } else {
        "opengrowth"
    }
}

fn write_all(shell: Shell, dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(dir)
        .with_context(|| format!("cannot create completion directory {}", dir.display()))?;
    for name in NAMES {
        let path = dir.join(file_name(shell, name));
        let mut file = fs::File::create(&path)
            .with_context(|| format!("cannot write completion file {}", path.display()))?;
        let mut command = crate::cli::Cli::command();
        clap_complete::generate(shell, &mut command, name, &mut file);
        println!("{}", path.display());
    }
    Ok(())
}

fn file_name(shell: Shell, name: &str) -> String {
    match shell {
        Shell::Bash => name.to_owned(),
        Shell::Zsh => format!("_{name}"),
        Shell::Fish => format!("{name}.fish"),
        Shell::PowerShell => format!("{name}.ps1"),
        Shell::Elvish => format!("{name}.elv"),
        _ => format!("{name}.completion"),
    }
}

fn detect_shell() -> Option<Shell> {
    let shell = std::env::var_os("SHELL")?;
    match Path::new(&shell).file_name()?.to_str()? {
        "bash" => Some(Shell::Bash),
        "zsh" => Some(Shell::Zsh),
        "fish" => Some(Shell::Fish),
        "elvish" => Some(Shell::Elvish),
        "pwsh" | "powershell" => Some(Shell::PowerShell),
        _ => None,
    }
}

fn default_dir(shell: Shell) -> anyhow::Result<PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("cannot find your home directory; pass --dir"))?;
    let xdg = |var: &str, fallback: &str| {
        std::env::var_os(var)
            .filter(|v| !v.is_empty())
            .map_or_else(|| home.join(fallback), PathBuf::from)
    };
    Ok(match shell {
        Shell::Bash => xdg("XDG_DATA_HOME", ".local/share").join("bash-completion/completions"),
        Shell::Zsh => home.join(".zfunc"),
        Shell::Fish => xdg("XDG_CONFIG_HOME", ".config").join("fish/completions"),
        Shell::PowerShell => home.join(".config/powershell/completions"),
        Shell::Elvish => home.join(".elvish/lib"),
        _ => home.join(".local/share/opengrowth/completions"),
    })
}

/// What the user still has to do, on stderr.
fn print_install_note(shell: Shell, dir: &Path) {
    match shell {
        Shell::Zsh => {
            eprintln!("Add this before `compinit` in ~/.zshrc if it is not already there:");
            eprintln!("  fpath=({} $fpath)", dir.display());
            eprintln!("Then restart zsh, or run `autoload -Uz compinit && compinit`.");
        }
        Shell::PowerShell => {
            eprintln!("Add these to your PowerShell profile if they are not already there:");
            for name in NAMES {
                eprintln!("  . {}", dir.join(file_name(shell, name)).display());
            }
        }
        Shell::Elvish => {
            eprintln!("Add these to ~/.elvish/rc.elv if they are not already there:");
            for name in NAMES {
                eprintln!("  use {name}");
            }
        }
        _ => eprintln!("Restart your shell to load the completions for `opengrowth` and `og`."),
    }
}
