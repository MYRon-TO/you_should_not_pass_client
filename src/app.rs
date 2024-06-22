use std::error;

use crossterm::event::KeyEvent;
use ratatui::{
    style::{Color, Style},
    Frame,
};

use crate::{
    tcp::{AccountList, Ack, Action},
    ui::running::RunningPage,
};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
pub enum Focus {
    List,
    Account,
    Name,
    Url,
    Note,
    Password,
}

/// Application.
#[derive(Debug)]
pub struct App {
    // pub login: RwLock<bool>,
    pub login: bool,
    pub focus: Focus,
    pub edit: bool,
    pub delete: bool,
    /// Is the application running?
    pub running: bool,
    pub account_list: AccountList,
    pub page: RunningPage<'static>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            // login: RwLock::new(false),
            login: false,
            running: true,
            focus: Focus::List,
            edit: false,
            delete: false,
            account_list: AccountList::default(),
            page: RunningPage::default(),
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl App {
    pub fn draw(&self, f: &mut Frame) {
        self.page.draw_running(f, self);
    }

    pub fn login_input(&mut self, key: KeyEvent) {
        self.page.login_textarea.input(key);
    }

    pub async fn sync(&mut self) {
        let conn = crate::tcp::Connect::new().await;
        let act = Action::GetInfo;
        let res = conn.request(act).await;
        if let Ok(Ack::Info { list }) = res {
            self.account_list.list = list;
        }
    }

    pub async fn login(&mut self) {
        let password = self
            .page
            .login_textarea
            .lines()
            .first()
            .unwrap_or(&"".to_string())
            .clone();

        let conn = crate::tcp::Connect::new().await;
        let act = Action::CheckIdentity { password };
        let res = conn.request(act).await;
        if let Ok(Ack::Ack) = res {
            self.login = true;
            self.sync().await;
        };
    }

    pub fn list_select_next_item(&mut self) {
        let len = self.account_list.list.len();
        if len == 0 {
            return;
        }
        if let Some(res) = self.account_list.selected.checked_add(1) {
            self.account_list.selected = res % len;
        }
    }

    pub fn list_select_before_item(&mut self) {
        let len = self.account_list.list.len();
        if len == 0 {
            return;
        }
        if let Some(res) = self.account_list.selected.checked_sub(1) {
            self.account_list.selected = res % len;
        } else {
            self.account_list.selected = len - 1;
        }
    }

    pub fn focus_next(&mut self) {
        self.focus = match self.focus {
            Focus::Account => {
                self.page
                    .account_textarea
                    .set_cursor_style(Style::default());
                self.page
                    .site_name_textarea
                    .set_cursor_style(Style::default().bg(Color::White));
                Focus::Name
            }
            Focus::Name => {
                self.page
                    .site_name_textarea
                    .set_cursor_style(Style::default());
                self.page
                    .password_textarea
                    .set_cursor_style(Style::default().bg(Color::White));
                Focus::Password
            }
            Focus::Password => {
                self.page
                    .password_textarea
                    .set_cursor_style(Style::default());
                self.page
                    .site_url_textarea
                    .set_cursor_style(Style::default().bg(Color::White));
                Focus::Url
            }
            Focus::Url => {
                self.page
                    .site_url_textarea
                    .set_cursor_style(Style::default());
                self.page
                    .note_textarea
                    .set_cursor_style(Style::default().bg(Color::White));
                Focus::Note
            }
            Focus::Note => {
                self.page.note_textarea.set_cursor_style(Style::default());
                self.page
                    .account_textarea
                    .set_cursor_style(Style::default().bg(Color::White));
                Focus::Account
            }
            _ => {
                self.page
                    .account_textarea
                    .set_cursor_style(Style::default().bg(Color::White));
                Focus::Account
            }
        }
    }

    pub fn new_item(&mut self) {
        self.page
            .account_textarea
            .set_cursor_style(Style::default());
        self.page
            .site_name_textarea
            .set_cursor_style(Style::default());
        self.page
            .site_url_textarea
            .set_cursor_style(Style::default());
        self.page
            .password_textarea
            .set_cursor_style(Style::default());
        self.page.note_textarea.set_cursor_style(Style::default());

        self.edit = false;
        self.focus_next();
    }

    pub async fn quit_edit(&mut self) {
        self.focus = Focus::List;

        let password = self
            .page
            .password_textarea
            .lines()
            .first()
            .unwrap_or(&"".to_string())
            .clone();
        let account = self
            .page
            .account_textarea
            .lines()
            .first()
            .unwrap_or(&"".to_string())
            .clone();
        let site_name = self
            .page
            .site_name_textarea
            .lines()
            .first()
            .unwrap_or(&"".to_string())
            .clone();
        let site_url = self
            .page
            .site_url_textarea
            .lines()
            .first()
            .unwrap_or(&"".to_string())
            .clone();

        let notes = self.page.note_textarea.lines();
        let mut note = String::new();
        note += &notes.first().unwrap().clone();
        for line in notes.iter().skip(1) {
            note = note + "\n" + line;
        }

        let action = if self.edit {
            let selected = self.account_list.selected;
            let id = self.account_list.list[selected].id;
            Action::ChangeWebsiteAccount {
                id: id.unwrap(),
                new_account: account,
                new_password: password,
                new_site_name: Some(site_name),
                new_site_url: site_url,
                new_note: Some(note),
            }
        } else {
            Action::AddWebsiteAccount {
                account,
                password,
                site_url,
                site_name: Some(site_name),
                note: Some(note),
            }
        };

        let conn = crate::tcp::Connect::new().await;
        let _ = conn.request(action).await;

        self.sync().await;

        self.page.password_textarea.select_all();
        self.page.password_textarea.delete_char();

        self.page.account_textarea.select_all();
        self.page.account_textarea.delete_char();

        self.page.site_name_textarea.select_all();
        self.page.site_name_textarea.delete_char();

        self.page.site_url_textarea.select_all();
        self.page.site_url_textarea.delete_char();

        self.page.note_textarea.select_all();
        self.page.note_textarea.delete_char();
    }

    pub fn edit(&mut self) {
        self.page
            .account_textarea
            .set_cursor_style(Style::default());
        self.page
            .site_name_textarea
            .set_cursor_style(Style::default());
        self.page
            .site_url_textarea
            .set_cursor_style(Style::default());
        self.page
            .password_textarea
            .set_cursor_style(Style::default());
        self.page.note_textarea.set_cursor_style(Style::default());

        self.edit = true;

        let selected = self.account_list.selected;
        self.page
            .account_textarea
            .insert_str(self.account_list.list[selected].account.clone());
        self.page
            .password_textarea
            .insert_str(self.account_list.list[selected].password.clone());
        self.page
            .site_url_textarea
            .insert_str(self.account_list.list[selected].site_url.clone());
        self.page.site_name_textarea.insert_str(
            self.account_list.list[selected]
                .site_name
                .clone()
                .unwrap_or("".to_string()),
        );
        self.page.note_textarea.insert_str(
            self.account_list.list[selected]
                .note
                .clone()
                .unwrap_or("".to_string()),
        );

        self.focus_next();
    }

    pub fn edit_input(&mut self, key: KeyEvent) {
        let textarea = match self.focus {
            Focus::Account => &mut self.page.account_textarea,
            Focus::Name => &mut self.page.site_name_textarea,
            Focus::Password => &mut self.page.password_textarea,
            Focus::Url => &mut self.page.site_url_textarea,
            Focus::Note => &mut self.page.note_textarea,
            _ => &mut self.page.account_textarea,
        };
        textarea.input(key);
    }

    pub fn visit(&self) {
        let selected = self.account_list.selected;
        let url = self.account_list.list[selected].site_url.clone();
        if webbrowser::open(&url).is_err() {
            println!("Failed to open the url");
        }
    }
    pub fn try_delete(&mut self) {
        self.delete = true;
    }

    pub fn cancel_delete(&mut self) {
        self.delete = false;
    }

    pub async fn delete(&mut self) {
        let selected = self.account_list.selected;
        let id = self.account_list.list[selected].id.unwrap();
        let action = Action::DeleteWebsiteAccount { website_id: id };

        let conn = crate::tcp::Connect::new().await;
        let ack = conn.request(action).await;
        if ack.is_ok() {
            self.sync().await;
        }
        self.delete = false;
    }
}
