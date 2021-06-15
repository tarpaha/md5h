use std::{io, env, path::Path, time::Instant};
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::Semaphore;
use clap::{Arg, App};

fn file_md5(filename: impl AsRef<Path>) -> io::Result<String> {
    let mut f = File::open(filename)?;
    let mut buffer = [0; 1 << 20];
    let mut context = md5::Context::new();
    loop {
        let n = f.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        context.consume(&buffer[..n]);
    }
    let digest = context.compute();
    Ok(format!("{:x}", digest))
}

fn get_files_recursively(path: impl AsRef<Path>) -> Vec<String> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().display().to_string())
        .sorted()
        .collect()
}

fn parse_args() -> (String, usize, bool) {
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

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let (folder, threads, quiet) = parse_args();

    if !quiet {
        println!("Running in folder {} with {} threads", folder, threads);
    }
    
    if !quiet {
        print!("Getting files list... ");
    }
    let files = get_files_recursively(folder);
    let files_count = files.len();
    if !quiet {
        println!("{} files found.", files_count);
    }

    let bar = ProgressBar::new(files_count as u64);
    if !quiet
    {
        bar.set_style(ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:.cyan/blue} {pos}/{len} eta {eta_precise}")
            .progress_chars("##-"));
        bar.tick();
        bar.enable_steady_tick(100);
    }

    let now = Instant::now();

    let semaphore = Arc::new(Semaphore::new(threads));
    let mut handles = Vec::new();
    for file in files {
        let bar = bar.clone();
        let semaphore = semaphore.clone();
        let handle = tokio::spawn(async move {
            let permit = semaphore.acquire().await.unwrap();
            let md5 = file_md5(&file)?;
            drop(permit);
            if !quiet {
                bar.inc(1);
            }
            Ok::<String, io::Error>(md5)
        });
        handles.push(handle);
    }

    let mut context = md5::Context::new();
    for handle in handles {
        let md5 = handle.await?.unwrap();
        context.consume(&md5);
    }
    let md5 = format!("{:x}", context.compute());
    
    if !quiet {
        bar.finish();
        println!("{} files in {} ms", files_count, now.elapsed().as_millis());
    }
    
    println!("{}{}", if !quiet {"MD5: "} else {""}, md5);

    Ok(())
}
