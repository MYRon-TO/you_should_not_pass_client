use std::sync::Arc;

use crate::app::{App, AppResult, Status};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::RwLock;

/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: Arc<RwLock<App>>) -> AppResult<()> {
    let mut app = app.write().await;
    match app.status {
        Status::Login => {
            match key_event.code {
                // Exit application on `Ctrl-C`
                KeyCode::Char('c') | KeyCode::Char('C') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        app.quit();
                    } else {
                        app.login_input(key_event);
                    }
                }
                KeyCode::Esc => {
                    app.quit();
                }
                KeyCode::Enter => {
                    app.login().await;
                }
                _ => app.login_input(key_event),
            }
        }
        Status::Delete => match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                app.delete().await;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.cancel_delete();
            }
            _ => {}
        },
        Status::Search => {
            match key_event.code {
                // Exit application on `ESC` or `q`
                KeyCode::Esc | KeyCode::Tab => {
                    app.search();
                }
                _ => {
                    app.search_input(key_event);
                }
            }
        }
        Status::List | Status::Edit => {
            match &app.focus {
                crate::app::Focus::List => {
                    // Handle key events when the focus is on the list.
                    match key_event.code {
                        // Exit application on `ESC` or `q`
                        KeyCode::Char('/') | KeyCode::Tab => {
                            app.search();
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            app.quit();
                        }
                        // Exit application on `Ctrl-C`
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            if key_event.modifiers == KeyModifiers::CONTROL {
                                app.quit();
                            }
                        }
                        KeyCode::Char('e') => {
                            app.edit();
                        }
                        KeyCode::Char('n') => {
                            app.new_item();
                        }
                        KeyCode::Char('j') => {
                            app.list_select_next_item();
                        }
                        KeyCode::Char('k') => {
                            app.list_select_before_item();
                        }
                        KeyCode::Char('d') => {
                            app.try_delete();
                        }
                        KeyCode::Enter => {
                            app.visit();
                        }
                        // Other handlers you could add here.
                        _ => {}
                    }
                }
                _ => {
                    // Handle key events when edit
                    match key_event.code {
                        // Exit application on `ESC` or `q`
                        KeyCode::Esc => {
                            app.quit_edit().await;
                        }
                        KeyCode::Tab => {
                            app.focus_next();
                        }
                        _ => {
                            app.edit_input(key_event);
                        }
                    }
                }
            }
        }
    };
    Ok(())
}
