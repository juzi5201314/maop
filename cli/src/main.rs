use std::path::PathBuf;

use argh::FromArgs;

use sub_commands::password::PasswordSubCommand;

mod sub_commands;

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
/// ...
enum SubCommandEnum {
    Password(PasswordSubCommand),
}

#[derive(FromArgs, Debug)]
/// Run maop
pub struct Run {
    #[argh(subcommand)]
    /// ...
    sub_command: Option<SubCommandEnum>,

    #[argh(option, short = 'c')]
    /// config files
    conf: Vec<String>,

    #[argh(option, short = 'e')]
    /// environment variable file
    env: Option<PathBuf>,

    #[argh(switch)]
    /// without password (login disabled)
    no_password: bool,
}

fn main() {
    let args = argh::from_env::<Run>();

    if let Some(env) = &args.env {
        dotenv::from_path(env).unwrap()
    }

    if let Some(sub_cmd) = &args.sub_command {
        match sub_cmd {
            SubCommandEnum::Password(cmd) => cmd.run(&args),
        }
    } else {
        core::run(args.conf, args.no_password);
    }
}
