use std::{io, path::Path, time::Instant};
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;
use itertools::Itertools;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::error::Error;
use tokio::sync::Semaphore;
use log::{info};

mod logger;
mod args;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    let (folder, threads, quiet) = args::parse();
    logger::init(quiet);
    
    info!("Running in folder {} with {} threads", folder, threads);
    info!("Getting files list... ");

    let files = get_files_recursively(folder);
    let files_count = files.len();

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
        let md5 = handle.await??;
        context.consume(&md5);
    }
    let md5 = format!("{:x}", context.compute());
    
    if !quiet {
        bar.finish();
    }
    info!("{} files in {} ms", files_count, now.elapsed().as_millis());
    println!("{}{}", if !quiet {"MD5: "} else {""}, md5);

    Ok(())
}
