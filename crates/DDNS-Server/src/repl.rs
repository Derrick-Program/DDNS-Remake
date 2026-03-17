use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

use crate::cli::Cli;
use crate::command::{AppState, CommandResult, handle};

fn parse_repl_line(input: &str) -> Result<Cli, String> {
    use clap::CommandFactory;
    let cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    let args = shlex::split(input).ok_or_else(|| "parse error: invalid quoting".to_string())?;

    let mut argv = Vec::with_capacity(args.len() + 1);
    argv.push(bin_name);
    argv.extend(args);

    Cli::try_parse_from(argv).map_err(|e| e.to_string())
}

pub async fn run(ctx: &Arc<AppState>) -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    loop {
        match rl.readline("DDNS-Server> ") {
            Ok(input) => {
                let input = input.trim();
                if input.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(input);
                match parse_repl_line(input) {
                    Ok(cli) => match handle(cli, ctx).await? {
                        CommandResult::Continue => {}
                        CommandResult::Exit => break,
                    },
                    Err(err) => eprintln!("{err}"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                eprintln!("readline error: {err}");
                break;
            }
        }
    }

    Ok(())
}
