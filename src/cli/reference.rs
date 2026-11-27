// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

//! The command-line reference in the documentation (`R56`): a page for each
//! command, holding the command's own `--help`, so that the pages say exactly
//! what the program says. `s/cli-docs` writes them, and `tests/cli_docs.rs`
//! fails while they are out of date.

use std::fmt::Write as _;

use clap::{Command, CommandFactory};

use super::Cli;

/// The directory under `docs/` that holds the pages.
pub const DIRECTORY: &str = "cli";

/// The width help is wrapped to, so that the pages do not depend on the
/// terminal they were written from.
const WIDTH: usize = 100;

const HEADER: &str = "<!-- Written by s/cli-docs from the program's own help (src/cli/reference.rs). Do not edit by hand. -->\n";

/// Each page, as its path under `docs/` and its contents.
///
/// # Panics
///
/// If clap's command tree is inconsistent, which its own debug checks
/// would show first.
#[must_use]
pub fn pages() -> Vec<(String, String)> {
    let mut cli = prepare(Cli::command().term_width(WIDTH));
    cli.build();
    let mut pages = vec![(format!("{DIRECTORY}/index.md"), index(&mut cli))];
    for name in documented(&cli) {
        let command = cli.find_subcommand_mut(&name).expect("a subcommand");
        pages.push((format!("{DIRECTORY}/{name}.md"), page(command)));
    }
    pages
}

/// Hides every argument's environment value, so that a variable set where
/// the pages are written, such as a local path, cannot reach them.
fn prepare(command: Command) -> Command {
    let command = command.mut_args(|arg| arg.hide_env_values(true));
    let names: Vec<String> = command
        .get_subcommands()
        .map(|subcommand| subcommand.get_name().to_owned())
        .collect();
    names.into_iter().fold(command, |command, name| {
        command.mut_subcommand(name, prepare)
    })
}

/// The subcommands with pages: all but `help` and any that are hidden.
fn documented(command: &Command) -> Vec<String> {
    command
        .get_subcommands()
        .filter(|subcommand| subcommand.get_name() != "help" && !subcommand.is_hide_set())
        .map(|subcommand| subcommand.get_name().to_owned())
        .collect()
}

/// A command's long help, without trailing spaces.
fn help(command: &mut Command) -> String {
    let text = command.render_long_help().to_string();
    let lines: Vec<&str> = text.lines().map(str::trim_end).collect();
    lines.join("\n").trim_end().to_owned()
}

fn title(command: &Command) -> String {
    command
        .get_bin_name()
        .unwrap_or_else(|| command.get_name())
        .to_owned()
}

fn index(cli: &mut Command) -> String {
    let mut text = format!(
        "{HEADER}\n# Command line\n\nopengrowth is one program, `opengrowth`, with a short alias, `og`. Each page here is a command's own `--help`.\n\n| Command | What it does |\n| --- | --- |\n"
    );
    for name in documented(cli) {
        let command = cli.find_subcommand(&name).expect("a subcommand");
        let about = command
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default();
        let _ = writeln!(text, "| [`{name}`]({name}.md) | {about} |");
    }
    let _ = write!(text, "\n```text\n{}\n```\n", help(cli));
    text
}

/// A command's page: its help, then each of its own subcommands' help.
fn page(command: &mut Command) -> String {
    let mut text = format!(
        "{HEADER}\n# {}\n\n```text\n{}\n```\n",
        title(command),
        help(command)
    );
    for name in documented(command) {
        let subcommand = command.find_subcommand_mut(&name).expect("a subcommand");
        let _ = write!(
            text,
            "\n## {}\n\n```text\n{}\n```\n",
            title(subcommand),
            help(subcommand)
        );
    }
    text
}
