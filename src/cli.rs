use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	#[command(subcommand)]
    pub command: Command,
}

/// Defines pmrs' subcommands
#[derive(Subcommand, Debug)]
pub enum Command {
	Start,
	Status,
}