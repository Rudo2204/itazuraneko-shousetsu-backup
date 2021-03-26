use anyhow::Result;
use csv::WriterBuilder;
use html_escape::decode_html_entities;
use scraper::{Html, Selector};

use crate::ShousetsuEntry;

pub fn serialize_data_to_csv_scraper(itazuraneko_data: &str) -> Result<String> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn xml_serialize_data() -> Result<()> {
        let input_html = std::fs::read_to_string("benches/sample.html")
            .expect("could not read sample itazuraneko data");
        let result_csv = std::fs::read_to_string("benches/sample_csv.csv")
            .expect("could not read sample_csv.csv");
        assert_eq!(serialize_data_to_csv_scraper(&input_html)?, result_csv);
        Ok(())
    }
}
