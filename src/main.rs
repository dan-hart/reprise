use clap::{CommandFactory, FromArgMatches};
use colored::{control::set_override, Colorize};
use is_terminal::IsTerminal;

use reprise::bitrise::BitriseClient;
use reprise::cli::args::{AppCommands, Cli, Commands, ConfigCommands};
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
    if reprise::cli::completions::handle_query()? {
        return Ok(());
    }
    let matches = Cli::command().get_matches();
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|error| error.exit());

    // Handle completions command early (no config or client needed)
    if let Commands::Completions(args) = &cli.command {
        reprise::cli::completions::run(args)?;
        return Ok(());
    }

    // Handle URL generation early (no config or client needed)
    if let Commands::Url(args) = &cli.command {
        if commands::is_generation_mode(args) {
            let explicit =
                matches.value_source("output") == Some(clap::parser::ValueSource::CommandLine);
            let format = if explicit {
                cli.output
            } else {
                let mut config = match reprise::config::Paths::new() {
                    Ok(paths) => Config::load_from(&paths)?,
                    Err(_) => Config::default(),
                };
                config.apply_context(&std::env::current_dir()?, cli.profile.as_deref())?;
                resolve_output_format(&config, cli.output, false)?
            };
            let output = commands::url_generate(args, format)?;
            if !output.is_empty() {
                println!("{output}");
            }
            return Ok(());
        }
    }

    // Load configuration
    let mut config = Config::load()?;
    config.network = reprise::bitrise::NetworkOptions {
        timeout: cli.timeout,
        retries: cli.retries,
        download_timeout: cli.download_timeout,
        search_limit: cli.search_limit,
        quiet: cli.quiet,
    };
    let profile_edit = matches!(&cli.command, Commands::Config(args) if matches!(&args.command,
        ConfigCommands::Profile { name: Some(_), token, app, format, r#use, remove }
            if token.is_some() || app.is_some() || format.is_some() || *r#use || *remove));
    if profile_edit {
        config.apply_profile_edit_context(cli.profile.as_deref())?;
    } else {
        config.apply_context(&std::env::current_dir()?, cli.profile.as_deref())?;
    }

    let format = resolve_output_format(
        &config,
        cli.output,
        matches.value_source("output") == Some(clap::parser::ValueSource::CommandLine),
    )?;

    if let Commands::Trigger(args) = &cli.command {
        if args
            .workflow
            .as_deref()
            .or(config.default_workflow())
            .is_none_or(|name| name.trim().is_empty())
        {
            return Err(RepriseError::InvalidArgument("the following required arguments were not provided: --workflow <WORKFLOW>. Pass --workflow or set workflow in .reprise.toml.".into()));
        }
    }

    if let Commands::Doctor(args) = &cli.command {
        let report = commands::doctor_report(&config, cli.token.as_deref(), args, format)?;
        if !report.output.is_empty() {
            println!("{}", report.output);
        }
        return report.failure.map_or(Ok(()), Err);
    }

    // Handle commands that don't need the API client
    let output = match &cli.command {
        Commands::Completions(_) => unreachable!(), // Handled above
        Commands::Config(args) => commands::config(&mut config, args, format)?,
        Commands::Doctor(args) => commands::doctor(&config, cli.token.as_deref(), args, format)?,
        Commands::View(args) => commands::view(&mut config, cli.token.as_deref(), args, format)?,

        // app show doesn't need API client
        Commands::App(args) if matches!(args.command, None | Some(AppCommands::Show)) => {
            commands::app_show(&config, format)?
        }

        // All other commands need the API client
        _ => {
            // Create client with inline token (CLI/env) or config file
            let client = match &cli.token {
                Some(token) => BitriseClient::with_options(token, config.network.clone())?,
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
                Commands::Yml(args) => commands::yml(&client, &config, args, format)?,
                Commands::Diagnose(args) => commands::diagnose(&client, &config, args, format)?,
                Commands::Compare(args) => commands::compare(&client, &config, args, format)?,
                Commands::Config(_) | Commands::Completions(_) => unreachable!(),
                Commands::Doctor(_) | Commands::View(_) => unreachable!(),
            }
        }
    };

    if !output.is_empty() {
        println!("{output}");
    }

    Ok(())
}

fn resolve_output_format(
    config: &Config,
    requested: reprise::cli::args::OutputFormat,
    explicit: bool,
) -> Result<reprise::cli::args::OutputFormat, RepriseError> {
    if explicit {
        return Ok(requested);
    }
    match config.output_format() {
        "pretty" => Ok(reprise::cli::args::OutputFormat::Pretty),
        "json" => Ok(reprise::cli::args::OutputFormat::Json),
        _ => Err(RepriseError::Config("Invalid configured output format; use --output pretty or --output json to override and repair the configuration".into())),
    }
}

#[cfg(test)]
mod output_tests {
    use super::*;
    use reprise::cli::args::OutputFormat;
    #[test]
    fn configured_output_is_used_unless_cli_is_explicit() {
        let mut config = Config::default();
        config.output.format = "json".into();
        assert_eq!(
            resolve_output_format(&config, OutputFormat::Pretty, false).unwrap(),
            OutputFormat::Json
        );
        assert_eq!(
            resolve_output_format(&config, OutputFormat::Pretty, true).unwrap(),
            OutputFormat::Pretty
        );
        config.output.format = "unsupported".into();
        assert!(resolve_output_format(&config, OutputFormat::Pretty, false).is_err());
    }
}
