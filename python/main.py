"""Main file."""
from pathlib import Path
import frontmatter
from dataclasses import dataclass
from typing import Any
from rich import print
from rich.console import Console
from rich.table import Table
from yattag import Doc


@dataclass
class MarkdownFile:
    file_name: str
    metadata: dict[Any, Any]
    content: str


BASE_PATH = Path(__file__).parent.parent

CONTENT = BASE_PATH / "content"

def get_files() -> list[type[MarkdownFile]]:
    """Scans for markdown files.

    Returns:
        list[type[MardownFile]]: List of markdown files.
    """
    markdownfiles = []
    for file in list(CONTENT.glob("**/*.md")):
        with open(file) as f:
            post = frontmatter.load(f)
            markdownfiles.append(MarkdownFile(file.name, post.metadata, post.content))
    return markdownfiles

def show_table(files: list[type[MarkdownFile]]) -> None:
    """Show files in a table printed to console.

    Args:
        files: list[type[MarkdownFile]]: List of markdown files.
    """
    table = Table(title="EyeSpace KB")

    table.add_column("Filename")
    table.add_column("Metadata")
    table.add_column("Content")

    for file in files:
        table.add_row(file.file_name,str(file.metadata),file.content)

    console = Console()
    console.print(table)


def make_html(files: list[type[MarkdownFile]]) -> None:
    """Create html documents from all the markdown file data.

    Args:
        files: list[type[MarkdownFile]]: List of markdown files.
    """
    doc, tag, text = Doc().tagtext()
    with tag('html'):
        with tag('body'):
            with tag("table"):
                with tag("thead"):
                    with tag("th"):
                        text("Filename")
                    with tag("th"):
                        text("Metadata")
                    with tag("th"):
                        text("Content")
                with tag("tbody"):
                    for file in files:
                        with tag("tr"):
                            with tag("td"):
                                text(file.file_name)
                            with tag("td"):
                                text(str(file.metadata))
                            with tag("td"):
                                text(file.content)

    with open("index.html", 'w') as f:
        f.write(doc.getvalue())



def main():
    files = get_files()
    show_table(files)
    make_html(files)

if __name__ == "__main__": 
    main()
