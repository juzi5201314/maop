use std::sync::Arc;

use chrono::SecondsFormat;
use colored::{ColoredString, Colorize};

use crate::{Level, Processor, Record};

macro_rules! _mod_and_pub {
    ($($mod_name:ident, $ps:ident),*) => {
        $(
            mod $mod_name;
            pub use $mod_name::$ps;
        )*
        pub fn default_processors() -> Vec<Box<dyn Processor<Output = anyhow::Result<()>> + Send + Sync>> {
            vec![
                $(
                    Box::new($ps::default()),
                )*
            ]
        }
    };
}

_mod_and_pub!(
    stdout_processors,
    StdoutProcessors,
    file_processors,
    FileProcessors
);

#[inline]
pub fn format(record: Arc<Record>) -> String {
    format!(
        "{time} [{lvl}] {debug}{content}\n",
        time = record
            .time
            .to_rfc3339_opts(SecondsFormat::Secs, true)
            .black()
            .on_bright_white(),
        lvl = level_color(record.level),
        debug = if record.level == Level::Debug {
            format!(
                "{}::{}:{} ",
                record.module_path, record.file, record.line
            )
            .purple()
        } else {
            ColoredString::default()
        },
        content = record.content,
    )
}

#[inline]
fn level_color(lvl: Level) -> ColoredString {
    (match lvl {
        Level::Debug => Colorize::green,
        Level::Info => Colorize::blue,
        Level::Warning => Colorize::yellow,
        Level::Error => Colorize::red,
    })(lvl.to_str())
}
