use argh::FromArgs;

#[derive(FromArgs)]
/// Run maop
pub struct Run {
    #[argh(option, short = 'c')]
    /// config files
    conf: Vec<String>
}

fn main() {
    let args = argh::from_env::<Run>();

    core::run(args.conf);
}
