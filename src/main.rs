use clap::Parser;
use colored::{control::set_override, Colorize};
use is_terminal::IsTerminal;

use reprise::bitrise::BitriseClient;
use reprise::cli::args::{AppCommands, Cli, Commands, CompletionsArgs};
use reprise::cli::commands;
use reprise::config::Config;
use reprise::error::RepriseError;

fn main() {
    // Respect NO_COLOR environment variable (https://no-color.org/)
    // Also disable colors when stdout is not a terminal (for piping)
    if std::env::var("NO_COLOR").is_ok() || !std::io::stdout().is_terminal() {
        set_override(false);
    }

    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red().bold(), e);
        std::process::exit(e.exit_code());
    }
}

fn run() -> Result<(), RepriseError> {
    let cli = Cli::parse();
    let format = cli.output;

    // Handle completions command early (no config or client needed)
    if let Commands::Completions(CompletionsArgs { shell }) = &cli.command {
        Cli::print_completions(*shell);
        return Ok(());
    }

    // Load configuration
    let mut config = Config::load()?;

    // Handle commands that don't need the API client
    let output = match &cli.command {
        Commands::Completions(_) => unreachable!(), // Handled above
        Commands::Config(args) => commands::config(&mut config, args, format)?,

        // app show doesn't need API client
        Commands::App(args) if matches!(args.command, None | Some(AppCommands::Show)) => {
            commands::app_show(&config, format)?
        }

        // All other commands need the API client
        _ => {
            // Create client with inline token (CLI/env) or config file
            let client = match &cli.token {
                Some(token) => BitriseClient::with_token(token)?,
                None => BitriseClient::new(&config)?,
            };

            match &cli.command {
                Commands::Apps(args) => commands::apps(&client, args, format)?,
                Commands::App(args) => commands::app_set(&client, &mut config, args, format)?,
                Commands::Builds(args) => commands::builds(&client, &config, args, format)?,
                Commands::Build(args) => commands::build(&client, &config, args, format)?,
                Commands::Log(args) => commands::log(&client, &config, args, format)?,
                Commands::Trigger(args) => commands::trigger(&client, &config, args, format)?,
                Commands::Artifacts(args) => commands::artifacts(&client, &config, args, format)?,
                Commands::Abort(args) => commands::abort(&client, &config, args, format)?,
                Commands::Url(args) => commands::url(&client, &mut config, args, format)?,
                Commands::Pipelines(args) => commands::pipelines(&client, &config, args, format)?,
                Commands::Pipeline(args) => commands::pipeline(&client, &config, args, format)?,
                Commands::Config(_) | Commands::Completions(_) => unreachable!(),
            }
        }
    };

    if !output.is_empty() {
        println!("{output}");
    }

    Ok(())
}
