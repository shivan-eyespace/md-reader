use camino::Utf8PathBuf;
use colored::Colorize;
use std::{collections::HashMap, ffi::OsStr, fs::read_to_string};
use walkdir::WalkDir;
use yaml_rust::{Yaml, YamlLoader};

pub struct MarkdownFile {
    pub name: String,
    pub frontmatter: HashMap<String, String>,
    pub content: String,
}

pub fn collect_files(path: Utf8PathBuf) -> Option<Vec<MarkdownFile>> {
    println!("Analysing path: {}", path.to_string().blue());
    match path.try_exists() {
        Ok(true) => println!("{}", "Valid path.".green()),
        _ => {
            println!("{}", "Invalid path.".red());
            return None;
        }
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
    if markdown_files.is_empty() {
        return None;
    }
    Some(markdown_files)
}

fn parse_frontmatter(text: &str) -> (HashMap<String, String>, String) {
    let mut yaml = HashMap::new();
    let content_body: &str;
    match text.starts_with("---\n") {
        true => {
            let after_marker = &text[4..];
            let end = after_marker.find("---\n");
            match end {
                Some(end) => {
                    let yaml_str = &text[4..(end + 4 as usize)];
                    yaml = parse_yaml(YamlLoader::load_from_str(yaml_str).unwrap());
                    content_body = &text[(end + 2 * 4 as usize)..]
                }
                None => content_body = &text,
            }
        }
        false => content_body = &text,
    }
    (yaml, content_body.to_string())
}

fn parse_yaml(frontmatter: Vec<Yaml>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("test".to_string(), "test".to_string());
    map
}
