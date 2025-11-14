use crate::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use similar::ChangeTag;

pub fn render_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header with file names
            Constraint::Min(0),    // Diff content or file browser
            Constraint::Length(3), // Status bar
        ])
        .split(f.area());

    // Header with file names
    render_header(f, app, chunks[0]);

    // Main content area - either diff view or file browser
    match app.mode {
        AppMode::DiffView => {
            render_diff_view(f, app, chunks[1]);
        }
        AppMode::SelectingSource | AppMode::SelectingTarget => {
            render_file_browser(f, app, chunks[1]);
        }
    }

    // Status bar
    render_status_bar(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled("Source: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&app.source_file),
        Span::raw("  "),
        Span::styled("Target: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(&app.target_file),
    ])])
    .block(Block::default().borders(Borders::ALL).title("Files"));

    f.render_widget(header, area);
}

fn render_diff_view(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let content_height = area.height.saturating_sub(2) as usize;
    let visible_lines: Vec<Line> = app
        .diff_lines
        .iter()
        .skip(app.scroll_offset)
        .take(content_height)
        .map(|diff_line| {
            let (prefix, style) = match diff_line.tag {
                ChangeTag::Delete => (
                    "-",
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::DIM),
                ),
                ChangeTag::Insert => (
                    "+",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::DIM),
                ),
                ChangeTag::Equal => (" ", Style::default()),
            };

            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(&diff_line.content, style),
            ])
        })
        .collect();

    let diff_widget = Paragraph::new(visible_lines)
        .block(Block::default().borders(Borders::ALL).title("Diff"))
        .wrap(Wrap { trim: false });

    f.render_widget(diff_widget, area);
}

fn render_file_browser(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let title = if app.mode == AppMode::SelectingSource {
        format!(
            "Select Source File - {}",
            app.file_browser.current_dir.display()
        )
    } else {
        format!(
            "Select Target File - {}",
            app.file_browser.current_dir.display()
        )
    };

    let content_height = area.height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = app
        .file_browser
        .entries
        .iter()
        .enumerate()
        .skip(app.file_browser.scroll_offset)
        .take(content_height)
        .map(|(idx, entry)| {
            let display_name = app.file_browser.get_display_name(entry);
            let style = if idx == app.file_browser.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(display_name).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, area);
}

fn render_status_bar(f: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let status_text = if let Some(ref msg) = app.status_message {
        vec![Line::from(Span::styled(
            msg,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ))]
    } else {
        match app.mode {
            AppMode::DiffView => vec![Line::from(vec![
                Span::raw("Commands: "),
                Span::styled("[q]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Quit  "),
                Span::styled("[s]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Select source  "),
                Span::styled("[t]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Select target  "),
                Span::styled("[c]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Copy  "),
                Span::styled("[e]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Export  "),
                Span::styled("[↑/↓]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Scroll"),
            ])],
            AppMode::SelectingSource | AppMode::SelectingTarget => vec![Line::from(vec![
                Span::styled("[↑/↓]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Navigate  "),
                Span::styled("[Enter]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Select  "),
                Span::styled("[Esc]", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" Cancel"),
            ])],
        }
    };

    let status_bar = Paragraph::new(status_text).block(Block::default().borders(Borders::ALL));

    f.render_widget(status_bar, area);
}
