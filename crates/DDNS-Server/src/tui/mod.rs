use crate::command::{AppState, utils};
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
};
use std::{io::stdout, sync::Arc};

// ── Top-level menu ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum MainMenu {
    Users,
    Devices,
    Domains,
    Quit,
}

impl MainMenu {
    const ALL: &'static [Self] = &[Self::Users, Self::Devices, Self::Domains, Self::Quit];
    fn label(&self) -> &str {
        match self {
            Self::Users => "  使用者管理  ",
            Self::Devices => "  裝置管理  ",
            Self::Domains => "  域名管理  ",
            Self::Quit => "  離開  ",
        }
    }
}

// ── Sub-menu actions ────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum SubAction {
    List,
    Add,
    Delete,
    Back,
}

impl SubAction {
    const ALL: &'static [Self] = &[Self::List, Self::Add, Self::Delete, Self::Back];
    fn label(&self) -> &str {
        match self {
            Self::List => "  列出  ",
            Self::Add => "  新增  ",
            Self::Delete => "  刪除  ",
            Self::Back => "  返回  ",
        }
    }
}

// ── Input form ──────────────────────────────────────────────────────────────

struct InputForm {
    labels: Vec<&'static str>,
    values: Vec<String>,
    password: Vec<bool>,
    focused: usize,
    title: &'static str,
    /// Pre-selected values not shown in the form (e.g. device_name for domain add)
    context: Vec<String>,
}

impl InputForm {
    fn new(title: &'static str, fields: &[(&'static str, bool)]) -> Self {
        Self {
            title,
            labels: fields.iter().map(|(l, _)| *l).collect(),
            values: vec![String::new(); fields.len()],
            password: fields.iter().map(|(_, p)| *p).collect(),
            focused: 0,
            context: Vec::new(),
        }
    }
    fn with_context(mut self, ctx: Vec<String>) -> Self {
        self.context = ctx;
        self
    }
    fn push_char(&mut self, c: char) {
        self.values[self.focused].push(c);
    }
    fn pop_char(&mut self) {
        self.values[self.focused].pop();
    }
    fn next_field(&mut self) {
        if self.focused + 1 < self.labels.len() {
            self.focused += 1;
        }
    }
    fn prev_field(&mut self) {
        self.focused = self.focused.saturating_sub(1);
    }
    fn is_last_field(&self) -> bool {
        self.focused == self.labels.len() - 1
    }
}

// ── List-picker (for delete) ─────────────────────────────────────────────────

struct ListPicker {
    title: &'static str,
    items: Vec<String>,
    state: ListState,
}

impl ListPicker {
    fn new(title: &'static str, items: Vec<String>) -> Self {
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }
        Self { title, items, state }
    }
    fn next(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let i = self.state.selected().map(|i| (i + 1).min(len - 1)).unwrap_or(0);
        self.state.select(Some(i));
    }
    fn prev(&mut self) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        let i = self.state.selected().map(|i| i.saturating_sub(1)).unwrap_or(0);
        self.state.select(Some(i));
    }
    fn selected_item(&self) -> Option<&str> {
        self.state.selected().and_then(|i| self.items.get(i)).map(String::as_str)
    }
}

// ── Screen stack ────────────────────────────────────────────────────────────

enum Screen {
    MainMenu(ListState),
    SubMenu {
        parent: MainMenu,
        state: ListState,
    },
    ShowList {
        title: String,
        lines: Vec<String>,
    },
    AddForm(InputForm),
    /// Step 1 of domain-add: pick a device from the list
    DomainDevicePick(ListPicker),
    DeletePick(ListPicker),
    Confirm {
        message: String,
        on_yes: Box<dyn FnOnce(&mut AppWrapper) -> String>,
    },
    Message(String), // result / info popup
}

// ── App wrapper ─────────────────────────────────────────────────────────────

struct AppWrapper {
    ctx: Arc<AppState>,
    screen_stack: Vec<Screen>,
    should_quit: bool,
}

impl AppWrapper {
    fn new(ctx: Arc<AppState>) -> Self {
        let mut main_state = ListState::default();
        main_state.select(Some(0));
        Self { ctx, screen_stack: vec![Screen::MainMenu(main_state)], should_quit: false }
    }

    fn push(&mut self, screen: Screen) {
        self.screen_stack.push(screen);
    }

    fn pop(&mut self) {
        if self.screen_stack.len() > 1 {
            self.screen_stack.pop();
        }
    }

    fn db(&self) -> crate::db::DbService {
        self.ctx.db_service.clone()
    }
}

// ── Key handling ─────────────────────────────────────────────────────────────

fn handle_key(app: &mut AppWrapper, code: KeyCode) {
    // We pop the current screen, operate, then push back or push new screen.
    // Use take-and-replace to avoid borrow issues.
    let screen = app.screen_stack.pop().unwrap();

    match screen {
        Screen::MainMenu(mut state) => {
            let len = MainMenu::ALL.len();
            match code {
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                    let i = state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                    state.select(Some(i));
                    app.screen_stack.push(Screen::MainMenu(state));
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                    let i = state.selected().map(|i| (i + len - 1) % len).unwrap_or(0);
                    state.select(Some(i));
                    app.screen_stack.push(Screen::MainMenu(state));
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let choice = state.selected().map(|i| MainMenu::ALL[i].clone());
                    app.screen_stack.push(Screen::MainMenu(state));
                    if let Some(choice) = choice {
                        match choice {
                            MainMenu::Quit => app.should_quit = true,
                            parent => {
                                let mut sub_state = ListState::default();
                                sub_state.select(Some(0));
                                app.push(Screen::SubMenu { parent, state: sub_state });
                            }
                        }
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    app.screen_stack.push(Screen::MainMenu(state));
                    app.should_quit = true;
                }
                _ => app.screen_stack.push(Screen::MainMenu(state)),
            }
        }

        Screen::SubMenu { parent, mut state } => {
            let len = SubAction::ALL.len();
            match code {
                KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                    let i = state.selected().map(|i| (i + 1) % len).unwrap_or(0);
                    state.select(Some(i));
                    app.screen_stack.push(Screen::SubMenu { parent, state });
                }
                KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                    let i = state.selected().map(|i| (i + len - 1) % len).unwrap_or(0);
                    state.select(Some(i));
                    app.screen_stack.push(Screen::SubMenu { parent, state });
                }
                KeyCode::Esc => {
                    // pop submenu, back to main
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let action = state.selected().map(|i| SubAction::ALL[i].clone());
                    app.screen_stack.push(Screen::SubMenu { parent: parent.clone(), state });
                    if let Some(action) = action {
                        match action {
                            SubAction::Back => {
                                app.pop(); // pop SubMenu
                            }
                            SubAction::List => open_list(app, &parent),
                            SubAction::Add => open_add_form(app, &parent),
                            SubAction::Delete => open_delete_pick(app, &parent),
                        }
                    }
                }
                _ => app.screen_stack.push(Screen::SubMenu { parent, state }),
            }
        }

        Screen::ShowList { title, lines } => {
            // any key → back
            let _ = (title, lines);
        }

        Screen::AddForm(mut form) => match code {
            KeyCode::Esc => { /* pop, discard */ }
            KeyCode::Tab | KeyCode::Down => {
                form.next_field();
                app.push(Screen::AddForm(form));
            }
            KeyCode::BackTab | KeyCode::Up => {
                form.prev_field();
                app.push(Screen::AddForm(form));
            }
            KeyCode::Enter => {
                if form.is_last_field() {
                    let msg = do_add(app, &form);
                    app.push(Screen::Message(msg));
                } else {
                    form.next_field();
                    app.push(Screen::AddForm(form));
                }
            }
            KeyCode::Backspace => {
                form.pop_char();
                app.push(Screen::AddForm(form));
            }
            KeyCode::Char(c) => {
                form.push_char(c);
                app.push(Screen::AddForm(form));
            }
            _ => app.push(Screen::AddForm(form)),
        },

        Screen::DomainDevicePick(mut picker) => match code {
            KeyCode::Esc => { /* pop */ }
            KeyCode::Down | KeyCode::Char('j') => {
                picker.next();
                app.push(Screen::DomainDevicePick(picker));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                picker.prev();
                app.push(Screen::DomainDevicePick(picker));
            }
            KeyCode::Enter => {
                if let Some(device) = picker.selected_item() {
                    let device = device.to_string();
                    app.push(Screen::DomainDevicePick(picker));
                    let form =
                        InputForm::new("新增域名", &[("域名", false)]).with_context(vec![device]);
                    app.push(Screen::AddForm(form));
                } else {
                    app.push(Screen::DomainDevicePick(picker));
                }
            }
            _ => app.push(Screen::DomainDevicePick(picker)),
        },

        Screen::DeletePick(mut picker) => match code {
            KeyCode::Esc => { /* pop */ }
            KeyCode::Down | KeyCode::Char('j') => {
                picker.next();
                app.push(Screen::DeletePick(picker));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                picker.prev();
                app.push(Screen::DeletePick(picker));
            }
            KeyCode::Enter => {
                if let Some(name) = picker.selected_item() {
                    let name = name.to_string();
                    // figure out which tab this picker belongs to by peeking at SubMenu below
                    let tab = app.screen_stack.iter().rev().find_map(|s| {
                        if let Screen::SubMenu { parent, .. } = s {
                            Some(parent.clone())
                        } else {
                            None
                        }
                    });
                    if let Some(parent) = tab {
                        let msg = format!("確定要刪除 '{name}' 嗎？");
                        app.push(Screen::DeletePick(picker));
                        app.push(Screen::Confirm {
                            message: msg,
                            on_yes: Box::new(move |w| do_delete(w, &parent, &name)),
                        });
                    } else {
                        app.push(Screen::DeletePick(picker));
                    }
                } else {
                    app.push(Screen::DeletePick(picker));
                }
            }
            _ => app.push(Screen::DeletePick(picker)),
        },

        Screen::Confirm { message: _, on_yes } => match code {
            KeyCode::Enter | KeyCode::Char('y') | KeyCode::Char('Y') => {
                let msg = on_yes(app);
                app.push(Screen::Message(msg));
            }
            _ => { /* pop confirm, discard */ }
        },

        Screen::Message(_) => { /* any key → pop */ }
    }
}

// ── Actions ──────────────────────────────────────────────────────────────────

fn open_list(app: &mut AppWrapper, parent: &MainMenu) {
    let mut db = app.db();
    let (title, lines) = match parent {
        MainMenu::Users => {
            let users = db.get_all_users().unwrap_or_default();
            ("使用者列表".to_string(), users)
        }
        MainMenu::Devices => {
            let devices = db.get_all_devices().unwrap_or_default();
            ("裝置列表".to_string(), devices)
        }
        MainMenu::Domains => {
            let rows = db.get_all_domains_with_device().unwrap_or_default();
            let lines = rows
                .into_iter()
                .map(|(dev, host, active)| {
                    format!("[{}] {}  ({})", if active { "✓" } else { "✗" }, host, dev)
                })
                .collect();
            ("域名列表".to_string(), lines)
        }
        MainMenu::Quit => return,
    };
    app.push(Screen::ShowList { title, lines });
}

fn open_add_form(app: &mut AppWrapper, parent: &MainMenu) {
    match parent {
        MainMenu::Users => {
            app.push(Screen::AddForm(InputForm::new(
                "新增使用者",
                &[("使用者名稱", false), ("密碼", true)],
            )));
        }
        MainMenu::Devices => {
            app.push(Screen::AddForm(InputForm::new(
                "新增裝置",
                &[("裝置名稱", false), ("擁有者使用者名稱", false)],
            )));
        }
        MainMenu::Domains => {
            // Step 1: pick a device from the list
            let devices = app.db().get_all_devices().unwrap_or_default();
            if devices.is_empty() {
                app.push(Screen::Message("目前沒有裝置，請先新增裝置".to_string()));
                return;
            }
            app.push(Screen::DomainDevicePick(ListPicker::new("選擇裝置", devices)));
        }
        MainMenu::Quit => {}
    }
}

fn open_delete_pick(app: &mut AppWrapper, parent: &MainMenu) {
    let mut db = app.db();
    let (title, items) = match parent {
        MainMenu::Users => ("選擇要刪除的使用者", db.get_all_users().unwrap_or_default()),
        MainMenu::Devices => ("選擇要刪除的裝置", db.get_all_devices().unwrap_or_default()),
        MainMenu::Domains => ("選擇要刪除的域名", db.get_all_domains().unwrap_or_default()),
        MainMenu::Quit => return,
    };
    if items.is_empty() {
        app.push(Screen::Message("目前沒有可刪除的項目".to_string()));
        return;
    }
    app.push(Screen::DeletePick(ListPicker::new(title, items)));
}

fn do_add(app: &AppWrapper, form: &InputForm) -> String {
    let mut db = app.db();
    let v = &form.values;
    match form.title {
        "新增使用者" => {
            let username = v[0].trim();
            let password = &v[1];
            if username.is_empty() || password.is_empty() {
                return "錯誤：欄位不能為空".into();
            }
            if db.find_user_by_username(username).ok().flatten().is_some() {
                return format!("錯誤：使用者 '{}' 已存在", username);
            }
            let hash = utils::hash_token(password);
            match db.create_user(username, &hash) {
                Ok(u) => format!("已新增使用者 '{}'", u.username),
                Err(e) => format!("錯誤：{e}"),
            }
        }
        "新增裝置" => {
            let device_name = v[0].trim();
            let owner = v[1].trim();
            if device_name.is_empty() || owner.is_empty() {
                return "錯誤：欄位不能為空".into();
            }
            if db.find_device_by_name(device_name).ok().flatten().is_some() {
                return format!("錯誤：裝置 '{}' 已存在", device_name);
            }
            let api_key = utils::generate_api_key();
            let token_hash = utils::hash_token(&api_key);
            let uuid = uuid::Uuid::new_v4();
            match db.create_device(owner, uuid, device_name.to_string(), token_hash) {
                Ok(d) => {
                    format!("已新增裝置 '{}'\n\nAPI Key（請妥善保存）:\n{}", d.device_name, api_key)
                }
                Err(e) => format!("錯誤：{e}"),
            }
        }
        "新增域名" => {
            // device_name comes from context (pre-selected), domain from v[0]
            let device_name = form.context.first().map(|s| s.as_str()).unwrap_or("").trim();
            let domain = v[0].trim();
            if device_name.is_empty() || domain.is_empty() {
                return "錯誤：欄位不能為空".into();
            }
            match db.find_device_by_name(device_name).ok().flatten() {
                None => format!("錯誤：裝置 '{}' 不存在", device_name),
                Some(dev) => match db.create_domain(dev.id, domain, true) {
                    Ok(d) => format!("已新增域名 '{}' → 裝置 '{}'", d.hostname, device_name),
                    Err(e) => format!("錯誤：{e}"),
                },
            }
        }
        _ => "未知表單".into(),
    }
}

fn do_delete(app: &mut AppWrapper, parent: &MainMenu, name: &str) -> String {
    let mut db = app.db();
    match parent {
        MainMenu::Users => match db.delete_user_by_username(name) {
            Ok(0) => format!("找不到使用者 '{}'", name),
            Ok(_) => format!("已刪除使用者 '{}'", name),
            Err(e) => format!("錯誤：{e}"),
        },
        MainMenu::Devices => match db.delete_device_by_name(name) {
            Ok(0) => format!("找不到裝置 '{}'", name),
            Ok(_) => format!("已刪除裝置 '{}'", name),
            Err(e) => format!("錯誤：{e}"),
        },
        MainMenu::Domains => match db.delete_domain_by_hostname(name) {
            Ok(0) => format!("找不到域名 '{}'", name),
            Ok(_) => format!("已刪除域名 '{}'", name),
            Err(e) => format!("錯誤：{e}"),
        },
        MainMenu::Quit => "無效操作".into(),
    }
}

// ── Rendering ────────────────────────────────────────────────────────────────

fn render(f: &mut Frame, app: &mut AppWrapper) {
    let area = f.area();

    // Background
    let bg = Block::default().style(Style::default().bg(Color::Blue));
    f.render_widget(bg, area);

    // Title bar
    let title_bar = Paragraph::new(" DDNS Server 管理介面 ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD));
    let title_area = Rect { x: area.x, y: area.y, width: area.width, height: 1 };
    f.render_widget(title_bar, title_area);

    // Hint bar at bottom
    let hint = Paragraph::new(" ↑↓/jk: 移動  Tab: 下一項  Enter: 選擇  Esc: 返回  q: 離開 ")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Black).bg(Color::Cyan));
    let hint_area = Rect { x: area.x, y: area.y + area.height - 1, width: area.width, height: 1 };
    f.render_widget(hint, hint_area);

    // Main content area
    let content =
        Rect { x: area.x, y: area.y + 1, width: area.width, height: area.height.saturating_sub(2) };

    // Render each screen layer in order (bottom-up)
    for screen in &app.screen_stack {
        if let Screen::MainMenu(state) = screen {
            render_main_menu(f, content, state)
        }
    }

    // Only render the top screen as overlay if it's not MainMenu
    match app.screen_stack.last_mut().unwrap() {
        Screen::MainMenu(_) => {}
        Screen::SubMenu { parent, state } => render_sub_menu(f, content, parent, state),
        Screen::ShowList { title, lines } => render_list_popup(f, content, title, lines),
        Screen::AddForm(form) => render_add_form(f, content, form),
        Screen::DomainDevicePick(picker) => render_list_picker(f, content, picker),
        Screen::DeletePick(picker) => render_list_picker(f, content, picker),
        Screen::Confirm { message, .. } => render_confirm(f, content, message),
        Screen::Message(msg) => render_message(f, content, msg),
    }
}

fn render_main_menu(f: &mut Frame, area: Rect, state: &ListState) {
    let items: Vec<ListItem> = MainMenu::ALL
        .iter()
        .map(|m| ListItem::new(Line::from(m.label()).alignment(Alignment::Center)))
        .collect();

    let w = 30u16;
    let h = (MainMenu::ALL.len() as u16) + 4;
    let menu_area = centered_rect(w, h, area);

    f.render_widget(Clear, menu_area);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 主選單 ")
                .title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .highlight_style(
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, menu_area, &mut state.clone());
}

fn render_sub_menu(f: &mut Frame, area: Rect, parent: &MainMenu, state: &ListState) {
    let items: Vec<ListItem> = SubAction::ALL
        .iter()
        .map(|a| ListItem::new(Line::from(a.label()).alignment(Alignment::Center)))
        .collect();

    let title = match parent {
        MainMenu::Users => " 使用者管理 ",
        MainMenu::Devices => " 裝置管理 ",
        MainMenu::Domains => " 域名管理 ",
        MainMenu::Quit => "",
    };

    let w = 30u16;
    let h = (SubAction::ALL.len() as u16) + 4;
    let menu_area = centered_rect(w, h, area);

    f.render_widget(Clear, menu_area);
    let list = List::new(items)
        .block(
            Block::default().borders(Borders::ALL).title(title).title_alignment(Alignment::Center),
        )
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .highlight_style(
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, menu_area, &mut state.clone());
}

fn render_list_popup(f: &mut Frame, area: Rect, title: &str, lines: &[String]) {
    let w = (area.width * 2 / 3).max(50);
    let h = (lines.len() as u16 + 4).min(area.height.saturating_sub(4)).max(6);
    let popup = centered_rect(w, h, area);

    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = if lines.is_empty() {
        vec![ListItem::new(Span::styled("（無項目）", Style::default().fg(Color::DarkGray)))]
    } else {
        lines.iter().map(|l| ListItem::new(l.as_str())).collect()
    };

    let hint = " 按任意鍵返回 ";
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .title_bottom(hint)
        .style(Style::default().bg(Color::Black).fg(Color::White));

    let list = List::new(items).block(block);
    f.render_widget(list, popup);
}

fn render_add_form(f: &mut Frame, area: Rect, form: &InputForm) {
    let n = form.labels.len();
    let w = 50u16;
    let h = (n as u16) * 3 + 4;
    let popup = centered_rect(w, h, area);

    f.render_widget(Clear, popup);

    let outer = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", form.title))
        .title_alignment(Alignment::Center)
        .title_bottom(" Tab: 下一欄  Enter: 確認  Esc: 取消 ")
        .style(Style::default().bg(Color::Black).fg(Color::White));
    f.render_widget(outer, popup);

    let inner = Rect {
        x: popup.x + 1,
        y: popup.y + 1,
        width: popup.width.saturating_sub(2),
        height: popup.height.saturating_sub(2),
    };

    let field_constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Length(3)).collect();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(field_constraints)
        .split(inner);

    for (i, label) in form.labels.iter().enumerate() {
        let focused = i == form.focused;
        let display = if form.password[i] {
            "•".repeat(form.values[i].len())
        } else {
            form.values[i].clone()
        };
        let text = if focused { format!("{display}▌") } else { display };
        let border_style = if focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };
        let para = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title(*label).border_style(border_style));
        f.render_widget(para, chunks[i]);
    }
}

fn render_list_picker(f: &mut Frame, area: Rect, picker: &mut ListPicker) {
    let w = (area.width / 2).max(40);
    let h = (picker.items.len() as u16 + 4).min(area.height.saturating_sub(4)).max(6);
    let popup = centered_rect(w, h, area);

    f.render_widget(Clear, popup);

    let items: Vec<ListItem> = picker.items.iter().map(|i| ListItem::new(i.as_str())).collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", picker.title))
                .title_alignment(Alignment::Center)
                .title_bottom(" Enter: 選擇  Esc: 取消 ")
                .style(Style::default().bg(Color::Black).fg(Color::White)),
        )
        .highlight_style(
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");
    f.render_stateful_widget(list, popup, &mut picker.state);
}

fn render_confirm(f: &mut Frame, area: Rect, message: &str) {
    let w = (message.len() as u16 + 6).max(40).min(area.width);
    let h = 5u16;
    let popup = centered_rect(w, h, area);

    f.render_widget(Clear, popup);
    let para = Paragraph::new(format!("\n  {message}")).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 確認 ")
            .title_alignment(Alignment::Center)
            .title_bottom(" Enter/y: 確認  其他鍵: 取消 ")
            .style(Style::default().bg(Color::Black).fg(Color::Yellow)),
    );
    f.render_widget(para, popup);
}

fn render_message(f: &mut Frame, area: Rect, msg: &str) {
    let lines: Vec<&str> = msg.lines().collect();
    let max_w = lines.iter().map(|l| l.len()).max().unwrap_or(30) as u16;
    let w = (max_w + 6).max(40).min(area.width);
    let h = (lines.len() as u16 + 4).max(5).min(area.height.saturating_sub(2));
    let popup = centered_rect(w, h, area);

    f.render_widget(Clear, popup);

    let content = lines.iter().map(|l| Line::from(format!("  {l}"))).collect::<Vec<_>>();
    let para = Paragraph::new(content).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" 訊息 ")
            .title_alignment(Alignment::Center)
            .title_bottom(" 按任意鍵繼續 ")
            .style(Style::default().bg(Color::Black).fg(Color::Green)),
    );
    f.render_widget(para, popup);
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let x = area.x + area.width.saturating_sub(width) / 2;
    let y = area.y + area.height.saturating_sub(height) / 2;
    Rect { x, y, width: width.min(area.width), height: height.min(area.height) }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run_tui(ctx: Arc<AppState>) -> Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppWrapper::new(ctx);

    let result = (|| -> Result<()> {
        loop {
            terminal.draw(|f| render(f, &mut app))?;
            if event::poll(std::time::Duration::from_millis(50))?
                && let Event::Key(key) = event::read()?
            {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                handle_key(&mut app, key.code);
            }
            if app.should_quit {
                break;
            }
        }
        Ok(())
    })();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
