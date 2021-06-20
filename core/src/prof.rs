use pprof::ProfilerGuard;
use std::fs::File;
use regex::Regex;

pub fn start<'a>() -> ProfilerGuard<'a> {
    pprof::ProfilerGuard::new(50).unwrap()
}

pub fn report(guard: &ProfilerGuard) {
    if let Ok(report) = guard.report().frames_post_processor(frames_post_processor()).build() {
        let mut options = pprof::flamegraph::Options::default();
        //options.image_width = Some(2500);
        options.count_name = "calls".to_string();
        let file = File::create("flamegraph.svg").unwrap();
        report.flamegraph_with_options(file, &mut options).unwrap();
    };
}

fn frames_post_processor() -> impl Fn(&mut pprof::Frames) {
    macro_rules! rename {
        ($from:expr, $to:expr) => {
            (Regex::new(concat!("^", $from, r"\d*$")).unwrap(), $to)
        };
    }
    let thread_rename = [
        (Regex::new(r"^<core::fut\d*$").unwrap(), "core-future"),
        rename!("figment", "figment"),
        rename!("nom", "nom"),
        rename!("serde_json", "serde_json"),
        rename!("once_cell", "once_cell"),
    ];

    move |frames| {
        for (regex, name) in thread_rename.iter() {
            if regex.is_match(&frames.thread_name) {
                frames.thread_name = name.to_string();
            }
        }
    }
}
