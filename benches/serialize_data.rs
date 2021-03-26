use anyhow::Result;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use csv::WriterBuilder;
use html_escape::decode_html_entities;
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::fs;

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

fn regex_serialize(itazuraneko_data: &str) -> Result<String> {
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

fn scraper_serialize(itazuraneko_data: &str) -> Result<String> {
    let fragment = Html::parse_document(&itazuraneko_data);
    let selector_a = Selector::parse("a").unwrap();
    let selector_td = Selector::parse("td").unwrap();

    let mut wtr = WriterBuilder::new()
        .delimiter(b';')
        .has_headers(false)
        .from_writer(vec![]);

    let mut id = String::new();
    let mut title = String::new();
    let mut author = String::new();
    let mut publisher = String::new();
    let mut epub_link = String::new();
    let mut epub_size = String::new();
    let mut publication_date = String::new();
    let mut count = 0;

    for (i, ele) in fragment.select(&selector_td).enumerate() {
        if count % 5 == 0 && i != 0 {
            wtr.serialize(ShousetsuEntry {
                id: decode_html_entities(&id).to_string(),
                title: decode_html_entities(&title).to_string(),
                author: decode_html_entities(&author).to_string(),
                publisher: decode_html_entities(&publisher).to_string(),
                epub_link: decode_html_entities(&epub_link).to_string(),
                epub_size: decode_html_entities(&epub_size).to_string(),
                publication_date: decode_html_entities(&publication_date).to_string(),
            })?;
        }
        if i % 5 == 0 {
            let tmp_frag = Html::parse_document(&ele.inner_html());
            let tmp = tmp_frag.select(&selector_a).next().unwrap();
            id = tmp
                .value()
                .attr("href")
                .expect("could not find link to itazuraneko website")
                .chars()
                .filter_map(|a| a.to_digit(10))
                .map(|c| c.to_string())
                .collect::<String>();
            title = tmp.inner_html();
        } else if i % 5 == 1 {
            author = ele.inner_html();
        } else if i % 5 == 2 {
            publisher = ele.inner_html();
        } else if i % 5 == 3 {
            //println!("epub link with size => {}", ele.inner_html());
            let tmp_frag = Html::parse_document(&ele.inner_html());
            let tmp = tmp_frag.select(&selector_a).next().unwrap();
            epub_link = tmp
                .value()
                .attr("href")
                .expect("could not find link to download epub file")
                .to_string();
            epub_size = tmp.inner_html();
        } else {
            publication_date = ele.inner_html();
        }
        count += 1
    }

    // Serialize the final element
    wtr.serialize(ShousetsuEntry {
        id: decode_html_entities(&id).to_string(),
        title: decode_html_entities(&title).to_string(),
        author: decode_html_entities(&author).to_string(),
        publisher: decode_html_entities(&publisher).to_string(),
        epub_link: decode_html_entities(&epub_link).to_string(),
        epub_size: decode_html_entities(&epub_size).to_string(),
        publication_date: decode_html_entities(&publication_date).to_string(),
    })?;

    let data = String::from_utf8(wtr.into_inner()?)?;

    Ok(data)
}

fn serialize_sample(c: &mut Criterion) {
    let sample_data =
        fs::read_to_string("benches/sample.html").expect("could not read sample.html");
    let mut group = c.benchmark_group("serialize_data");
    group.bench_with_input(
        BenchmarkId::new("regex", &sample_data),
        &sample_data,
        |b, i| {
            b.iter(|| {
                regex_serialize(i).unwrap();
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("scraper", &sample_data),
        &sample_data,
        |b, i| {
            b.iter(|| {
                scraper_serialize(i).unwrap();
            })
        },
    );

    group.finish();
}

criterion_group!(benches, serialize_sample);
criterion_main!(benches);
