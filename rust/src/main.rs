use camino::Utf8PathBuf;
use colored::Colorize;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    env::args,
    ffi::OsStr,
    fs::read_to_string,
    io, thread,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::Layout,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use walkdir::WalkDir;
use yaml_rust::{scanner::ScanError, Yaml, YamlLoader};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i <= 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i))
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

struct App<'a> {
    items: StatefulList<(&'a str, usize)>,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            items: StatefulList::with_items(vec![("Item0", 1), ("Item1", 2)]),
        }
    }
}

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

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => app.items.unselect(),
                    KeyCode::Down => app.items.next(),
                    KeyCode::Up => app.items.previous(),
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now()
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default();
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|i| {
            let mut lines = vec![Spans::from(i.0)];
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    f.render_stateful_widget(
        items,
        Layout::default().split(f.size())[0],
        &mut app.items.state,
    );
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
