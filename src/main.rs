use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg};
use csv::ReaderBuilder;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::process::Command;

mod scraper;

#[derive(Debug, Serialize, Deserialize)]
struct ShousetsuEntry {
    id: String,
    title: String,
    author: String,
    publisher: String,
    epub_link: String,
    epub_size: String,
    publication_date: String,
}

fn main() -> Result<()> {
    let matches = App::new("itazuraneko_backup")
        .setting(AppSettings::DisableHelpSubcommand)
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(
            App::new("download").about("download data").arg(
                Arg::with_name("input")
                    .help("use prepared csv file")
                    .short("i")
                    .long("input")
                    .value_name("CSV")
                    .takes_value(true),
            ),
        )
        .subcommand(
            App::new("export")
                .about("export data for inspection purpose")
                .arg(
                    Arg::with_name("input")
                        .help("use prepared csv file")
                        .short("i")
                        .long("input")
                        .value_name("CSV")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("csv")
                        .help("export itazuraneko data to csv file")
                        .short("s")
                        .long("csv")
                        .conflicts_with("job")
                        .conflicts_with("input"),
                )
                .arg(
                    Arg::with_name("job")
                        .help("export jobs file for running parallel")
                        .short("j")
                        .long("job")
                        .conflicts_with("csv"),
                )
                .arg(
                    Arg::with_name("output")
                        .help("output file")
                        .short("o")
                        .long("output")
                        .value_name("OUTPUT")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("export") {
        if matches.is_present("csv") {
            download_shousetsu_index()?;
            let output_file = matches.value_of("output").unwrap_or("itazuraneko.csv");
            parse_and_save_csv("index.html", &output_file)?;
            println!("everything is finished!");
        } else if matches.is_present("job") {
            let output_file = matches.value_of("output").unwrap_or("jobs.txt");
            if let Some(csv_path) = matches.value_of("input") {
                parse_csv_and_save_jobs(&csv_path, &output_file)?;
            } else {
                download_shousetsu_index()?;
                // TODO: make tempdir here so itazuraneko.csv file gets cleaned up after
                parse_and_save_csv("index.html", "itazuraneko.csv")?;
                parse_csv_and_save_jobs("itazuraneko.csv", &output_file)?;
            }
            println!("everything is finished!");
        } else {
            eprintln!("you need to specific either --csv or --job!");
        }
    } else if let Some(ref matches) = matches.subcommand_matches("download") {
        if let Some(csv_path) = matches.value_of("input") {
            download_from_path(csv_path)?;
        } else {
            download_shousetsu_index()?;
            parse_and_save_csv("index.html", "itazuraneko.csv")?;
            download_from_path("itazuraneko.csv")?;
        }
    }

    Ok(())
}

fn parse_and_save_csv(input_path: &str, output_path: &str) -> Result<()> {
    let s = fs::read_to_string(input_path)?;
    let csv_string = scraper::serialize_data_to_csv_scraper(&s)?;
    let mut csv_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path)?;

    csv_file.write_all(csv_string.as_bytes())?;

    Ok(())
}

fn download_from_path(csv_path: &str) -> Result<()> {
    // ensure "download" directory is created first
    fs::create_dir_all("download")?;

    let data = fs::read_to_string(csv_path)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(data.as_bytes());
    let iter = rdr.deserialize();
    let mut entry_vec: Vec<ShousetsuEntry> = Vec::new();
    for entry in iter {
        let record: ShousetsuEntry = entry?;
        entry_vec.push(record);
    }

    let entry_number = entry_vec.len();
    println!(
        "I found {} entries from the csv file. Starting download process...",
        entry_number
    );
    // TODO: Figure out which id actually fails
    // probably need to change download_single_file instead of here
    let _download_parallel: Vec<_> = entry_vec
        .par_iter()
        .progress_count(entry_vec.len() as u64)
        .map(|entry| download_single_file(entry))
        .filter_map(|x| x.err())
        .collect();

    let num_file_downloaded = fs::read_dir("download")?.count();
    if entry_number != num_file_downloaded {
        println!(
            "There were {} entries in total, but only {} download",
            entry_number, num_file_downloaded
        );
        println!(
            "There were {} entries in total, but only {} download",
            entry_number, num_file_downloaded
        );
    } else {
        println!("Everything is finished!");
    }

    Ok(())
}

// TODO: catch panic
// https://stackoverflow.com/questions/59091329/how-do-i-catch-a-panic-from-rayons-par-iter
fn download_single_file(entry: &ShousetsuEntry) -> Result<()> {
    let _status = Command::new("monolith")
        .arg("-s") // Be quiet!
        .arg(format!("https://yonde.itazuraneko.org/novelhtml/{}.html", entry.id).as_str())
        .arg("-o")
        .arg(format!("download/{}.html", entry.id,).as_str())
        .status()
        .unwrap();

    Ok(())
}

fn download_shousetsu_index() -> Result<()> {
    Command::new("monolith")
        .arg("-s") // Be quiet!
        .arg("https://yonde.itazuraneko.org/other/kensaku.html")
        .arg("-o")
        .arg("index.html")
        .status()
        .unwrap();

    Ok(())
}

fn serialize_csv_to_jobs(csv_path: &str) -> Result<String> {
    let data = fs::read_to_string(csv_path)?;
    let mut rdr = ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_reader(data.as_bytes());
    let iter = rdr.deserialize();

    let mut ret_string = String::new();
    for entry in iter {
        let record: ShousetsuEntry = entry?;
        ret_string.push_str(
            format!(
                "monolith -s https://yonde.itazuraneko.org/novelhtml/{}.html -o download/{}.html\n",
                record.id, record.id
            )
            .as_str(),
        )
    }
    Ok(ret_string)
}

fn parse_csv_and_save_jobs(input_path: &str, output_path: &str) -> Result<()> {
    let job_string = serialize_csv_to_jobs(&input_path)?;
    let mut job_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path)?;

    job_file.write_all(job_string.as_bytes())?;

    Ok(())
}
