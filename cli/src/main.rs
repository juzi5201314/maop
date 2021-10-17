use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs)]
/// Run maop
pub struct Run {
    #[argh(option, short = 'c')]
    /// config files
    conf: Vec<String>,

    #[argh(option, short = 'e')]
    /// environment variable file
    env: Option<PathBuf>,
}

fn main() {
    let args = argh::from_env::<Run>();

    if let Some(env) = args.env {
        dotenv::from_path(env).unwrap()
    }

    core::run(args.conf);
}
