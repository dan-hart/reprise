//! Deterministic command-reference generation from the actual Clap schema.
use super::args::Cli;
use clap::CommandFactory;

pub fn markdown() -> String {
    let mut output = String::from("# Command reference\n\nGenerated from Clap. Regenerate with `cargo run --example generate_reference`. Do not edit by hand.\n\n");
    render(Cli::command(), "reprise", &mut output);
    output
}

fn render(mut command: clap::Command, path: &str, output: &mut String) {
    command = command.bin_name(path);
    command.build();
    output.push_str(&format!(
        "## {path}\n\n```text\n{}\n```\n\n",
        command.render_long_help()
    ));
    for subcommand in command.get_subcommands().filter(|c| c.get_name() != "help") {
        if !subcommand.is_hide_set() {
            let child_path = format!("{path} {}", subcommand.get_name());
            render(subcommand.clone(), &child_path, output);
        }
    }
}
