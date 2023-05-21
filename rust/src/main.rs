use camino::Utf8PathBuf;
use colored::Colorize;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env::args, ffi::OsStr, fs::read_to_string, io, thread, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use walkdir::WalkDir;
use yaml_rust::{scanner::ScanError, Yaml, YamlLoader};

struct MarkdownFile {
    name: String,
    frontmatter: Vec<Yaml>,
    content: String,
}

fn main() {
    let arguments: Vec<String> = args().collect();
    let argument = arguments.get(1);
    let path;
    match argument {
        Some(a) => path = Utf8PathBuf::from(a),
        None => path = Utf8PathBuf::from(arguments.get(0).expect("Unexpected error!")),
    };
    println!("Analysing path: {}", path.to_string().blue());
    match path.try_exists() {
        Ok(true) => println!("{}", "Valid path.".green()),
        _ => {
            println!("{}", "Invalid path.".red());
            return;
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
        println!("{}", name);
        let (frontmatter, content) = parse_frontmatter(&file_content);

        let markdown_file = MarkdownFile {
            name,
            frontmatter,
            content,
        };
        markdown_files.push(markdown_file)
    }
    println!(
        "Markdown file found: {}.",
        markdown_files.len().to_string().blue()
    );
    _ = draw(markdown_files).unwrap();
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

fn draw(files: Vec<MarkdownFile>) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let items: Vec<ListItem> = files
        .iter()
        .map(|e| ListItem::new(e.name.to_string()))
        .collect();
    terminal.draw(|f| {
        let size = f.size();
        let list = List::new(items)
            .style(Style::default().fg(Color::White))
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">>");
        f.render_widget(list, size);
    })?;

    thread::sleep(Duration::from_millis(5000));

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

// TODO commit to github
// Make a menu system - search?
// Can go into md file, see the yaml frontmatter
