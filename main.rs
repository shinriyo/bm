use std::fs;
use std::io;
use std::path::PathBuf;
use std::process;

use crossterm::event::{self, Event, KeyCode};
use dirs::home_dir;
use ratatui::{backend::CrosstermBackend, layout::*, style::*, widgets::*, Terminal};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Bookmark {
    name: String,
    path: String,
}

#[derive(Serialize, Deserialize)]
struct BookmarkFile {
    bookmarks: Vec<Bookmark>,
}

fn get_bookmark_path() -> PathBuf {
    home_dir().unwrap().join(".bm/bookmarks.toml")
}

fn load_bookmarks() -> Vec<Bookmark> {
    let path = get_bookmark_path();
    if path.exists() {
        let content = fs::read_to_string(path).unwrap_or_default();
        toml::from_str::<BookmarkFile>(&content).map(|f| f.bookmarks).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_bookmarks(bookmarks: &[Bookmark]) {
    let path = get_bookmark_path();
    let dir = path.parent().unwrap();
    fs::create_dir_all(dir).unwrap();
    let data = toml::to_string(&BookmarkFile { bookmarks: bookmarks.to_vec() }).unwrap();
    fs::write(path, data).unwrap();
}

fn main() -> Result<(), io::Error> {
    let mut bookmarks = load_bookmarks();
    let mut selected = 0;

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Min(1)].as_ref())
                .split(f.size());

            let items: Vec<ListItem> = bookmarks
                .iter()
                .map(|b| ListItem::new(b.path.clone()))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Bookmarks"))
                .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
                .highlight_symbol("→ ");

            f.render_stateful_widget(list, chunks[0], &mut tui::widgets::ListState::default().select(Some(selected)));
        })?;

        if event::poll(std::time::Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('j') => {
                        if selected + 1 < bookmarks.len() {
                            selected += 1;
                        }
                    }
                    KeyCode::Char('k') => {
                        if selected > 0 {
                            selected -= 1;
                        }
                    }
                    KeyCode::Char('u') => {
                        let cwd = std::env::current_dir().unwrap().to_str().unwrap().to_string();
                        if !bookmarks.iter().any(|b| b.path == cwd) {
                            bookmarks.push(Bookmark {
                                name: format!("bookmark_{}", bookmarks.len() + 1),
                                path: cwd,
                            });
                            save_bookmarks(&bookmarks);
                        }
                    }
                    KeyCode::Char('!') => {
                        // 簡略化: 選択中のものを削除（確認なし）
                        if !bookmarks.is_empty() {
                            bookmarks.remove(selected);
                            if selected >= bookmarks.len() && selected > 0 {
                                selected -= 1;
                            }
                            save_bookmarks(&bookmarks);
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(b) = bookmarks.get(selected) {
                            println!("{}", b.path);
                            process::exit(0);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    terminal.clear()?;
    Ok(())
}

