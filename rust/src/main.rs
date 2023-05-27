use camino::Utf8PathBuf;
use md_reader::file_reader::collect_files;
use md_reader::interface::draw;
use std::env::args;

fn main() {
    let arguments: Vec<String> = args().collect();
    let argument = arguments.get(1);
    let path;
    match argument {
        Some(a) => path = Utf8PathBuf::from(a),
        None => path = Utf8PathBuf::from(arguments.get(0).expect("Unexpected error!")),
    };
    let markdown_files = collect_files(path);
    _ = draw(
        markdown_files
            .iter()
            .enumerate()
            .map(|e| (e.1.name.as_str(), e.0))
            .collect(),
    )
    .unwrap();
}
