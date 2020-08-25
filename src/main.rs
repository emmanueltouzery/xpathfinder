use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufReader;
use xml::common::{Position, TextPosition};

use xml::reader::{EventReader, XmlEvent};

fn main() {
    let mut args = env::args().skip(1);
    let (filename, xpath) = match (args.next(), args.next()) {
        (Some(f), Some(path)) => (f, path),
        _ => {
            eprintln!("arguments: <xml filename> <xpath>");
            std::process::exit(1);
        }
    };
    let file = File::open(filename).unwrap();
    let file = BufReader::new(file);
    match parse_xpath(&xpath) {
        Ok(parsed_path) => print_pos(find_pos(file, &parsed_path)),
        Err(e) => {
            eprintln!("Error parsing xpath: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_pos(pos: Result<Option<TextPosition>, String>) {
    match pos {
        Ok(Some(pos)) => println!("Found xpath at position: {}", pos),
        Ok(None) => println!("Did not find xpath in document"),
        Err(e) => eprintln!("Error: {}", e),
    }
}

fn parse_xpath(xpath: &str) -> Result<Vec<(String, usize)>, String> {
    let normalized_xpath = if xpath.starts_with('/') {
        &xpath[1..]
    } else {
        xpath
    };
    normalized_xpath
        .split('/')
        .map(|item| {
            // an item should be like "tag[2]"
            // this block should extract ("tag", 2) from it
            let elements = if item.ends_with(']') {
                // first get ["tag", "2"] out of it
                Some(item[..item.len() - 1].split('[').collect::<Vec<_>>())
            } else {
                None
            };
            // check there are two elements and that the second element is a number...
            match elements.as_deref() {
                Some(&[path, count]) => count
                    .parse::<usize>()
                    .map(|c| (path.to_string(), c))
                    .map_err(|e| {
                        format!(
                            "failed parsing xpath at section: {}: {}",
                            item,
                            e.to_string()
                        )
                    }),
                _ => Err(format!("failed parsing xpath at section: {}", item)),
            }
        })
        .collect()
}

fn find_pos(
    reader: impl std::io::Read,
    needle_xpath: &[(String, usize)],
) -> Result<Option<TextPosition>, String> {
    let mut parser = EventReader::new(reader);
    let mut cur_xpath = vec![];
    let mut seen_xpaths = HashSet::new();
    let normalized_needle = if needle_xpath.last() == Some(&("text()".to_string(), 1)) {
        &needle_xpath[0..needle_xpath.len() - 1]
    } else {
        needle_xpath
    };
    loop {
        match parser.next() {
            Ok(XmlEvent::StartElement { name, .. }) => {
                // try to find the current xpath.
                // we need to find the final index, eg div[1] div[2] and so on.
                let mut i = 0;
                loop {
                    i += 1;
                    // let's try tag[i]
                    cur_xpath.push((name.to_string(), i));
                    if seen_xpaths.contains(&cur_xpath) {
                        // i've already seen tag[i] => we'll have
                        // to try tag[i+1]. Remove the last element
                        // and repeat the loop
                        cur_xpath.pop();
                    } else {
                        // yes, i this is the good path, stop searching
                        break;
                    }
                }
                seen_xpaths.insert(cur_xpath.clone());
                if cur_xpath == normalized_needle {
                    return Ok(Some(parser.position()));
                }
            }
            Ok(XmlEvent::EndElement { name: _ }) => {
                cur_xpath.pop();
            }
            Err(e) => {
                return Err(format!("XML parse error: {}", e));
            }
            Ok(XmlEvent::EndDocument) => return Ok(None),
            _ => {}
        }
    }
}

#[test]
fn parse_xpath_should_work() {
    assert_eq!(
        Ok(vec![("a".to_string(), 1), ("b".to_string(), 2)]),
        parse_xpath("a[1]/b[2]")
    );
}

#[test]
fn parse_xpath_should_work_also_with_a_leader_slash_and_trailing_text() {
    assert_eq!(
        Ok(vec![
            ("a".to_string(), 1),
            ("b".to_string(), 2),
            ("text()".to_string(), 1)
        ]),
        parse_xpath("/a[1]/b[2]/text()[1]")
    );
}

#[test]
fn parse_xpath_should_report_errors() {
    assert_eq!(
        Err(
            "failed parsing xpath at section: b[]: cannot parse integer from empty string"
                .to_string()
        ),
        parse_xpath("a[1]/b[]")
    );
}

#[test]
fn finds_xpath_should_report_the_correct_position() {
    assert_eq!(
        Ok(Some(TextPosition { row: 0, column: 14 })),
        find_pos(
            "<a><b><c/></b><b/></a>".as_bytes(),
            &vec![("a".to_string(), 1), ("b".to_string(), 2)]
        )
    );
}

#[test]
fn finds_xpath_should_return_none_if_no_match() {
    assert_eq!(
        Ok(None),
        find_pos(
            "<a><b><c/></b><b/></a>".as_bytes(),
            &vec![("a".to_string(), 1), ("b".to_string(), 4)]
        )
    );
}

#[test]
fn finds_xpath_should_return_err_if_invalid_xml() {
    assert_eq!(
        Err("XML parse error: 1:16 Unexpected token: />".to_string()),
        find_pos(
            "<a><b><c/></b>b/></a>".as_bytes(),
            &vec![("a".to_string(), 1), ("b".to_string(), 4)]
        )
    );
}

#[test]
fn finds_xpath_should_ignore_trailing_text_segment() {
    assert_eq!(
        Ok(Some(TextPosition { row: 0, column: 14 })),
        find_pos(
            "<a><b><c/></b><b/></a>".as_bytes(),
            &vec![
                ("a".to_string(), 1),
                ("b".to_string(), 2),
                ("text()".to_string(), 1)
            ]
        )
    );
}
