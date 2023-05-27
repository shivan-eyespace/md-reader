use camino::Utf8PathBuf;
use colored::Colorize;
use std::{ffi::OsStr, fs::read_to_string};
use walkdir::WalkDir;
use yaml_rust::{Yaml, YamlLoader};

pub struct MarkdownFile {
    pub name: String,
    pub frontmatter: Vec<Yaml>,
    pub content: String,
}

pub fn collect_files(path: Utf8PathBuf) -> Vec<MarkdownFile> {
    // TODO: need to do something when there are no files.
    println!("Analysing path: {}", path.to_string().blue());
    match path.try_exists() {
        Ok(true) => println!("{}", "Valid path.".green()),
        _ => {
            println!("{}", "Invalid path.".red());
            panic!();
        }
    }
    match path.is_dir() {
        true => println!("Searching directory."),
        false => println!("Only single file detected."),
    }
    let mut markdown_files: Vec<MarkdownFile> = Vec::new();
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some(OsStr::new("md")))
    {
        let file_path = entry.path();
        let file_content = read_to_string(file_path).unwrap();
        let name = file_path.file_name().unwrap().to_str().unwrap().to_string();
        let (frontmatter, content) = parse_frontmatter(&file_content);

        let markdown_file = MarkdownFile {
            name,
            frontmatter,
            content,
        };
        markdown_files.push(markdown_file)
    }
    markdown_files
}

fn parse_frontmatter(text: &str) -> (Vec<Yaml>, String) {
    let mut yaml: Vec<Yaml> = vec![];
    let content_body: &str;
    match text.starts_with("---\n") {
        true => {
            let after_marker = &text[4..];
            let end = after_marker.find("---\n");
            match end {
                Some(end) => {
                    let yaml_str = &text[4..(end + 4 as usize)];
                    yaml = YamlLoader::load_from_str(yaml_str).unwrap();
                    content_body = &text[(end + 2 * 4 as usize)..]
                }
                None => content_body = &text,
            }
        }
        false => content_body = &text,
    }
    (yaml, content_body.to_string())
}
