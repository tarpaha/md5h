use clap::{Arg, App};

pub fn parse() -> (String, usize, bool) {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(Arg::with_name("folder")
            .help("Folder to get MD5 from")
            .required(true))
        .arg(Arg::with_name("threads")
            .help("Number of threads, by default equals to cpu count")
            .short("t")
            .long("threads")
            .value_name("count")
            .takes_value(true))
        .arg(Arg::with_name("quiet")
            .help("Quiet mode, only prints resulting MD5")
            .short("q")
            .long("quiet")
            .takes_value(false))
        .after_help(format!("Usage example: \"md5h .\"\n\
                             Repository: {}", env!("CARGO_PKG_REPOSITORY")).as_str())
        .get_matches();

    (
        matches.value_of("folder").unwrap().parse().unwrap(),
        match matches.value_of("threads") {
            None => num_cpus::get(),
            Some(threads) => threads.parse().unwrap()
        },
        matches.is_present("quiet")
    )
}
