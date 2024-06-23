use crate::{
    app::{App, Status},
    tcp::AccountList,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use tui_textarea::TextArea;

#[derive(Debug)]
pub struct RunningPage<'a> {
    pub login_textarea: TextArea<'a>,
    pub account_textarea: TextArea<'a>,
    pub password_textarea: TextArea<'a>,
    pub site_name_textarea: TextArea<'a>,
    pub site_url_textarea: TextArea<'a>,
    pub note_textarea: TextArea<'a>,
    pub search_textarea: TextArea<'a>,
}

impl Default for RunningPage<'_> {
    fn default() -> Self {
        let mut login_textarea = TextArea::default();
        login_textarea.set_cursor_line_style(Style::default());
        login_textarea.set_mask_char('\u{2022}');
        login_textarea.set_placeholder_text("Enter your password");
        login_textarea.set_block(Block::default().borders(Borders::ALL).title("Password"));

        let mut account_textarea = TextArea::default();
        account_textarea.set_block(Block::default().borders(Borders::ALL).title("Account"));
        account_textarea.set_cursor_line_style(Style::default());
        account_textarea.set_cursor_style(Style::default());

        let mut site_name_textarea = TextArea::default();
        site_name_textarea.set_block(Block::default().borders(Borders::ALL).title("Site Name"));
        site_name_textarea.set_cursor_line_style(Style::default());
        site_name_textarea.set_cursor_style(Style::default());

        let mut site_url_textarea = TextArea::default();
        site_url_textarea.set_block(Block::default().borders(Borders::ALL).title("Site URL"));
        site_url_textarea.set_cursor_line_style(Style::default());
        site_url_textarea.set_cursor_style(Style::default());

        let mut note_textarea = TextArea::default();
        note_textarea.set_block(Block::default().borders(Borders::ALL).title("Note"));
        note_textarea.set_cursor_line_style(Style::default());
        note_textarea.set_cursor_style(Style::default());

        let mut password_textarea = TextArea::default();
        password_textarea.set_block(Block::default().borders(Borders::ALL).title("Password"));
        password_textarea.set_cursor_line_style(Style::default());
        password_textarea.set_cursor_style(Style::default());

        let mut search_textarea = TextArea::default();
        search_textarea.set_block(Block::default().borders(Borders::ALL).title("Search"));
        search_textarea.set_cursor_line_style(Style::default());
        search_textarea.set_cursor_style(Style::default());

        Self {
            login_textarea,
            account_textarea,
            password_textarea,
            site_name_textarea,
            site_url_textarea,
            note_textarea,
            search_textarea,
        }
    }
}

impl RunningPage<'_> {
    pub fn draw_running(&self, f: &mut Frame, app: &App) {
        // let is_login = app.login;
        let status = &app.status;
        let account_list = &app.account_list;

        match status {
            Status::Login => {
                self.draw_login(f);
            }
            Status::Delete => {
                let account = account_list.list[account_list.selected].account.clone();
                let site_name = match &account_list.list[account_list.selected].site_name {
                    Some(name) => name.clone(),
                    None => account_list.list[account_list.selected].site_url.clone(),
                };

                self.draw_delete(f, account, site_name);
            }
            Status::List | Status::Search => {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(61), Constraint::Percentage(39)])
                    .split(f.size());

                let menu_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ])
                    .split(chunks[0]);

                let detail_chunks = chunks[1];
                let title_chunk = menu_chunks[0];
                let search_chunk = menu_chunks[1];
                let list_chunk = menu_chunks[2];

                let search_text = if self.search_textarea.lines().join("").is_empty() {
                    None
                } else {
                    Some(self.search_textarea.lines().join(""))
                };

                self.draw_title(f, title_chunk);
                self.draw_search(f, search_chunk);
                self.draw_list(f, list_chunk, account_list, search_text);
                self.draw_detail(f, detail_chunks, account_list);
            }
            Status::Edit => {
                self.draw_edit(f);
            }
        }
    }

    fn draw_search(&self, f: &mut Frame, area: Rect) {
        f.render_widget(self.search_textarea.widget(), area);
    }

    fn draw_delete(&self, f: &mut Frame, account: String, site_name: String) {
        let area = self.centered_single_line_rect(60, f.size());
        // f.render_widget(popup_block, area);

        let layout = Layout::default()
            .margin(1)
            .constraints([Constraint::Min(1)])
            .split(area);

        let text = Text::styled(
            format!("Delete {} - {}? [Y/N]", account, site_name),
            Style::default(),
        );
        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .title("Delete")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .alignment(Alignment::Center);
        f.render_widget(paragraph, layout[0]);
    }

    fn draw_edit(&self, f: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(5),
                Constraint::Max(5),
                Constraint::Max(10),
                Constraint::Max(10),
                Constraint::Min(5),
            ])
            .split(f.size());

        let account_area = layout[0];
        let site_name_area = layout[1];
        let password_area = layout[2];
        let site_url_area = layout[3];
        let note_area = layout[4];

        f.render_widget(
            // account_paragraph.block(block.clone().title("Account")),
            self.account_textarea.widget(),
            account_area,
        );
        f.render_widget(
            // account_paragraph.block(block.clone().title("Account")),
            self.password_textarea.widget(),
            password_area,
        );
        f.render_widget(
            // site_url_paragraph.block(block.clone().title("Site URL")),
            self.site_url_textarea.widget(),
            site_url_area,
        );
        f.render_widget(
            // site_name_paragraph.block(block.clone().title("Site Name")),
            self.site_name_textarea.widget(),
            site_name_area,
        );
        f.render_widget(
            // note_paragraph.block(block.clone().title("Note")),
            self.note_textarea.widget(),
            note_area,
        );
    }

    fn draw_login(&self, f: &mut Frame) {
        // let popup_block = Block::default()
        //     .borders(Borders::NONE)
        //     .border_type(BorderType::Rounded)
        //     .style(Style::default().bg(Color::Black));

        let area = self.centered_single_line_rect(60, f.size());
        // f.render_widget(popup_block, area);

        let layout = Layout::default()
            .margin(1)
            .constraints([Constraint::Min(1)])
            .split(area);

        f.render_widget(self.login_textarea.widget(), layout[0]);
    }

    fn draw_title(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            // .title("You Should Not Pass")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        let title = Paragraph::new(Text::styled("You Should Not Pass", Style::default()))
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(title, area);
    }

    fn draw_list(
        &self,
        f: &mut Frame,
        area: Rect,
        account_list: &AccountList,
        search_text: Option<String>,
    ) {
        let mut list_items = Vec::<ListItem>::new();
        for item in &account_list.list {
            let account = item.account.clone();
            let name = match &item.site_name {
                Some(name) => name,
                None => &item.site_url.clone(),
            };

            let is_dead = item.is_dead;

            let text = format!("{} - {}", account, name);
            if let Some(search_text) = &search_text {
                if !text.to_lowercase().contains(&search_text.to_lowercase()) {
                    continue;
                }
            }

            list_items.push(ListItem::new(Line::from(if is_dead {
                text.crossed_out().italic().dark_gray()
            } else {
                text.white()
            })));
        }

        let list = List::new(list_items)
            .block(
                Block::default()
                    .title("Account List")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .highlight_symbol("> ")
            .highlight_style(Style::default().fg(Color::Yellow));

        let selected = account_list.selected;

        f.render_stateful_widget(
            list,
            area,
            &mut ListState::default().with_selected(Some(selected)),
        );
    }

    fn draw_detail(&self, f: &mut Frame, area: Rect, account_list: &AccountList) {
        let selected = account_list.selected;

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Max(5),
                Constraint::Max(10),
                Constraint::Max(10),
                Constraint::Min(5),
            ])
            .split(area);

        let account_area = layout[0];
        let site_name_area = layout[1];
        let site_url_area = layout[2];
        let note_area = layout[3];

        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default())
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded);

        // f.render_widget(block, account_area);
        // f.render_widget(block, site_name_area);
        // f.render_widget(block, site_url_area);
        // f.render_widget(block, note_area);

        let item = account_list.list.get(selected);

        if let Some(i) = item {
            let account = i.account.clone();
            let site_url = i.site_url.clone();
            let site_name = match &i.site_name {
                Some(name) => name.clone(),
                None => site_url.clone(),
            };
            let note = match &i.note {
                Some(note) => note.clone(),
                None => "".to_string(),
            };

            let account_paragraph = Paragraph::new(Text::styled(account, Style::default()));
            let site_url_paragraph = Paragraph::new(Text::styled(site_url, Style::default()));
            let site_name_paragraph = Paragraph::new(Text::styled(site_name, Style::default()));
            let note_paragraph = Paragraph::new(Text::styled(note, Style::default()));

            // self.account_textarea.insert_str(account);
            // self.site_url_textarea.insert_str(site_url);
            // self.site_name_textarea.insert_str(site_name);
            // self.note_textarea.insert_str(note);

            f.render_widget(
                account_paragraph.block(block.clone().title("Account")),
                // self.account_textarea.widget(),
                account_area,
            );
            f.render_widget(
                site_url_paragraph.block(block.clone().title("Site URL")),
                // self.site_url_textarea.widget(),
                site_url_area,
            );
            f.render_widget(
                site_name_paragraph.block(block.clone().title("Site Name")),
                // self.site_name_textarea.widget(),
                site_name_area,
            );
            f.render_widget(
                note_paragraph.block(block.clone().title("Note")),
                // self.note_textarea.widget(),
                note_area,
            );
        } else {
            let detail = Paragraph::new(Text::styled("No item selected", Style::default()))
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(detail, area);
        };
    }
    /// helper function to create a centered rect using up certain percentage of the available rect `r`
    fn _centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        // Then cut the middle vertical piece into three width-wise pieces
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1] // Return the middle chunk
    }

    fn centered_single_line_rect(&self, percent_x: u16, r: Rect) -> Rect {
        // Cut the given rectangle into three vertical pieces
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Max(5), Constraint::Min(1)])
            .split(r);

        // Then cut the middle vertical piece into three width-wise pieces
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1] // Return the middle chunk
    }
}
