use anyhow::Result;
use clap::{crate_authors, crate_description, crate_version, App, AppSettings, Arg};
use csv::{ReaderBuilder, WriterBuilder};
use html_escape::decode_html_entities;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::process::Command;

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
        .arg(
            Arg::with_name("input")
                .help("use prepared csv file")
                .short("i")
                .long("input")
                .value_name("CSV")
                .takes_value(true),
        )
        .subcommand(
            App::new("export_csv")
                .about("export csv for inspection")
                .arg(
                    Arg::with_name("output")
                        .help("output csv file")
                        .short("o")
                        .long("output")
                        .value_name("OUTPUT")
                        .default_value("itazuraneko.csv")
                        .takes_value(true),
                ),
        )
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("export_csv") {
        download_shousetsu_index()?;
        let csv_path = matches.value_of("output").expect("should never fail");
        parse_and_save_csv("index.html", &csv_path)?;
        return Ok(());
    }

    if let Some(csv_path) = matches.value_of("csv") {
        download_from_path(csv_path)?;
    } else {
        download_shousetsu_index()?;
        parse_and_save_csv("index.html", "itazuraneko.csv")?;
        download_from_path("itazuraneko.csv")?;
    }

    Ok(())
}

fn parse_and_save_csv(input_path: &str, output_path: &str) -> Result<()> {
    let s = fs::read_to_string(input_path)?;
    let csv_string = serialize_data_to_csv(&s)?;
    let mut csv_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(output_path)?;

    csv_file.write_all(csv_string.as_bytes())?;

    Ok(())
}

fn serialize_data_to_csv(itazuraneko_data: &str) -> Result<String> {
    let re = Regex::new(
        r#"<tr><td><a href="https://yonde.itazuraneko.org/novelhtml/(\d+)\.html">(.*?)</a></td><td>(.*?)</td><td>(.*?)</td><td><a href="(.*?)">(.*?)</a></td><td>(.*?)</td></tr>"#,
    )?;

    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_writer(vec![]);

    for cap in re.captures_iter(itazuraneko_data) {
        wtr.serialize(ShousetsuEntry {
            id: decode_html_entities(&cap[1]).to_string(),
            title: decode_html_entities(&cap[2]).to_string(),
            author: decode_html_entities(&cap[3]).to_string(),
            publisher: decode_html_entities(&cap[4]).to_string(),
            epub_link: decode_html_entities(&cap[5]).to_string(),
            epub_size: decode_html_entities(&cap[6]).to_string(),
            publication_date: decode_html_entities(&cap[7]).to_string(),
        })?;
    }

    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(data)
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
