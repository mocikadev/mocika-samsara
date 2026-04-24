pub mod cli;
pub mod commands;
pub mod config;
pub mod error;
pub mod git;
pub mod i18n;
pub mod knowledge;

use clap::Parser;
use cli::{Cli, Command};
use config::Config;
use error::SamsaraError;

pub fn run() -> Result<(), SamsaraError> {
    i18n::init_from_env();
    let cli = Cli::parse();
    let config = Config::new(cli.home, cli.dry_run);
    match cli.command {
        Command::Init(args) => commands::init::run(args, &config),
        Command::Write(args) => commands::write::run(args, &config),
        Command::Search(args) => commands::search::run(args, &config),
        Command::Promote(args) => commands::promote::run(args, &config),
        Command::Domain(args) => commands::domain::run(args, &config),
        Command::Archive(args) => commands::archive::run(args, &config),
        Command::Lint(args) => commands::lint::run(args, &config),
        Command::Status(args) => commands::status::run(args, &config),
        Command::Log(args) => commands::log::run(args, &config),
        Command::Prime(args) => commands::prime::run(args, &config),
        Command::Demote(args) => commands::demote::run(args, &config),
        Command::Remote(args) => commands::remote::run(args, &config),
        Command::Reflect(args) => commands::reflect::run(args, &config),
        Command::SkillNote(args) => commands::skill_note::run(args, &config),
        Command::Push => commands::push::run(&config),
        Command::Pull => commands::pull::run(&config),
        Command::SelfUpdate(args) => commands::self_update::run(args, &config),
        Command::Mcp(args) => commands::mcp::run(args, &config),
    }
}
