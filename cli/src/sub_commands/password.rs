use argh::FromArgs;

use http::PASSWORD_FILE_NAME;

use crate::Run;

#[derive(FromArgs, PartialEq, Debug)]
/// change password
#[argh(subcommand, name = "password")]
pub struct PasswordSubCommand {
    #[argh(positional)]
    /// password
    password: String,
}

impl PasswordSubCommand {
    pub fn run(&self, args: &Run) {
        config::init(args.conf.iter().map(|s| s.into()).collect())
            .expect("config error");
        http::set_password(
            &config::get_config_temp()
                .data_path()
                .join(PASSWORD_FILE_NAME),
            self.password.clone(),
        )
        .unwrap();
        println!("password is set");
    }
}
