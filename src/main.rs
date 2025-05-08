use std::fs;
use std::io;
use std::path::PathBuf;
use std::error::Error;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use dirs::home_dir;
use ratatui::{
    backend::CrosstermBackend,
    layout::*,
    style::*,
    text::Span,
    widgets::*,
    Terminal,
};
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

fn run_tui() -> Result<(), Box<dyn Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut bookmarks = load_bookmarks();
    let mut selected = 0;

    let result = loop {
        terminal.draw(|f| {
            let size = f.area();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(size);

            let items: Vec<ListItem> = bookmarks
                .iter()
                .map(|b| ListItem::new(b.path.clone()))
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Bookmarks"))
                .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
                .highlight_symbol("→ ");

            let mut state = ListState::default();
            state.select(Some(selected));
            f.render_stateful_widget(list, chunks[0], &mut state);

            // Help message at bottom
            let help_text = "j/k: move  u: add bookmark  !: delete  Enter: select  q: quit";
            let help = Span::raw(help_text);
            f.render_widget(
                Block::default()
                    .title(help)
                    .borders(Borders::BOTTOM),
                chunks[1],
            );
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => break Ok(()),
                    KeyCode::Char('j') | KeyCode::Down => {
                        selected = (selected + 1).min(bookmarks.len().saturating_sub(1));
                    }
                    KeyCode::Char('k') | KeyCode::Up => {
                        selected = selected.saturating_sub(1);
                    }
                    KeyCode::Char('u') => {
                        if let Ok(cwd) = std::env::current_dir() {
                            if let Some(cwd_str) = cwd.to_str() {
                                let path = cwd_str.to_string();
                                if !bookmarks.iter().any(|b| b.path == path) {
                                    bookmarks.push(Bookmark {
                                        name: format!("bookmark_{}", bookmarks.len() + 1),
                                        path: path.clone(),
                                    });
                                    save_bookmarks(&bookmarks);
                                    selected = bookmarks.len() - 1;
                                }
                            }
                        }
                    }
                    KeyCode::Char('!') => {
                        if !bookmarks.is_empty() {
                            let mut show_confirm = true;
                            while show_confirm {
                                terminal.draw(|f| {
                                    let size = f.area();
                                    let chunks = Layout::default()
                                        .direction(Direction::Vertical)
                                        .constraints([
                                            Constraint::Min(3),
                                            Constraint::Length(3),
                                        ])
                                        .split(size);

                                    // Bookmark list
                                    let items: Vec<ListItem> = bookmarks
                                        .iter()
                                        .map(|b| ListItem::new(b.path.clone()))
                                        .collect();

                                    let list = List::new(items)
                                        .block(Block::default().borders(Borders::ALL).title("Bookmarks"))
                                        .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black))
                                        .highlight_symbol("→ ");

                                    let mut state = ListState::default();
                                    state.select(Some(selected));
                                    f.render_stateful_widget(list, chunks[0], &mut state);

                                    // Confirmation dialog
                                    let confirm = Paragraph::new("Delete this bookmark? (y/n)")
                                        .block(Block::default().borders(Borders::ALL).title("Confirm"))
                                        .style(Style::default().fg(Color::Yellow));
                                    f.render_widget(confirm, chunks[1]);
                                })?;

                                if let Event::Key(key) = event::read()? {
                                    if key.kind == KeyEventKind::Press {
                                        match key.code {
                                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                                bookmarks.remove(selected);
                                                if selected >= bookmarks.len() && selected > 0 {
                                                    selected -= 1;
                                                }
                                                save_bookmarks(&bookmarks);
                                                show_confirm = false;
                                            }
                                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                                show_confirm = false;
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(b) = bookmarks.get(selected) {
                            println!("{}", b.path);
                            break Ok(());
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    // Cleanup
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )?;

    result
}

fn main() -> Result<(), Box<dyn Error>> {
    run_tui()
}
