use crate::frontend::state::{State, TablesSort};
use crate::indexer::IndexedTable;
use crate::simplefrontend::capitalize_first_letter;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::style::palette::tailwind::{AMBER, CYAN, SLATE};
use ratatui::style::Modifier;
use ratatui::text::Line;
use ratatui::widgets::{HighlightSpacing, ListItem, Wrap};
use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use std::collections::HashSet;
use std::time::SystemTime;

const GRAY: Color = Color::Rgb(100, 100, 100);

const LIST_SELECTED_STYLE: Style = Style::new()
    .bg(SLATE.c800)
    .fg(CYAN.c500)
    .add_modifier(Modifier::BOLD);

const INFO_ITEM_HEADER_STYLE: Style = Style::new().fg(CYAN.c500);

const KEY_BINDING_STYLE: Style = Style::new().fg(AMBER.c500);

pub fn render(state: &mut State, f: &mut Frame) {
    let chunks = Layout::new(
        Direction::Vertical,
        [Constraint::Fill(1), Constraint::Length(1)],
    )
    .direction(Direction::Vertical)
    .margin(1)
    .split(f.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(chunks[0]);

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = state.tables.items.iter().map(ListItem::from).collect();

    let sorting = match state.tables.sort {
        TablesSort::Name => "Alphabetical",
        TablesSort::LastModified => "Last Modified",
    };
    let title =
        Span::from("Tables") + Span::from(format!(" ({}) ", sorting)).add_modifier(Modifier::DIM);
    let items_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title);
    let tables = ratatui::widgets::List::new(items)
        .block(items_block)
        .highlight_symbol("> ")
        .highlight_spacing(HighlightSpacing::Always)
        .highlight_style(LIST_SELECTED_STYLE);
    let tables_scrollbar = ratatui::widgets::Scrollbar::default().style(Style::default());

    let paragraph_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title("Table Info");
    let selected = state.tables.state.selected();
    let paragraph_text = match selected {
        Some(i) => {
            let table = &state.tables.items[i];
            table_to_paragraph(table, &state.roms)
        }
        None => Text::from("No table selected").style(Style::default().italic()),
    };
    let paragraph = Paragraph::new(paragraph_text)
        .wrap(Wrap { trim: true })
        .block(paragraph_block);

    // Table List
    f.render_stateful_widget(tables, main_chunks[0], &mut state.tables.state);
    f.render_stateful_widget(
        tables_scrollbar,
        main_chunks[0],
        &mut state.tables.vertical_scroll_state,
    );

    // Table Info
    f.render_widget(paragraph, main_chunks[1]);

    render_key_bindings(state, f, chunks[1]);

    //dialog(state, f);
}

/// Renders the key bindings.
pub fn render_key_bindings(state: &mut State, frame: &mut Frame, rect: Rect) {
    let chunks = Layout::vertical([Constraint::Percentage(100), Constraint::Min(1)]).split(rect);
    let key_bindings = state.get_key_bindings();
    let line = Line::from(
        key_bindings
            .iter()
            .enumerate()
            .flat_map(|(i, (keys, desc))| {
                vec![
                    "[".fg(GRAY),
                    Span::from(*keys).style(KEY_BINDING_STYLE),
                    " → ".fg(GRAY),
                    Span::from(*desc),
                    "]".fg(GRAY),
                    if i != key_bindings.len() - 1 { " " } else { "" }.into(),
                ]
            })
            .collect::<Vec<Span>>(),
    );
    frame.render_widget(Paragraph::new(line.alignment(Alignment::Center)), chunks[1]);
}

fn table_to_paragraph<'a>(table: &IndexedTable, roms: &HashSet<String>) -> Text<'a> {
    // table name rendered as header
    // centered bold table name
    let table_name = table.displayed_name();
    let name_line =
        Line::from(table_name).style(Style::default().add_modifier(Modifier::BOLD).fg(AMBER.c500));
    let name_text = Text::from(name_line);

    let warnings: Vec<Line> = table
        .warnings(roms)
        .iter()
        .map(|w| Line::styled(format!("⚠️ {}", w), Style::default().fg(AMBER.c500)))
        .collect();
    let warning_text = Text::from(warnings);

    let path_line = Span::from("Path:          ").style(INFO_ITEM_HEADER_STYLE)
        + Span::from(table.path.display().to_string());
    let game_name_line = table
        .game_name
        .clone()
        .map(|n| Span::from("Game Name:     ").style(INFO_ITEM_HEADER_STYLE) + Span::from(n))
        .unwrap_or_default();
    let rom_line = table
        .local_rom_path
        .clone()
        .map(|p| {
            Span::from("Rom Path:      ").style(INFO_ITEM_HEADER_STYLE)
                + Span::from(p.display().to_string())
        })
        .unwrap_or_default();
    let b2s_line = table
        .b2s_path
        .clone()
        .map(|p| {
            Span::from("B2S Path:      ").style(INFO_ITEM_HEADER_STYLE)
                + Span::from(p.display().to_string())
        })
        .unwrap_or_default();
    let f = timeago::Formatter::new();
    let time: SystemTime = table.last_modified.into();
    let duration = time.elapsed().unwrap();
    let last_modified_human_readable = f.convert(duration);
    let last_modified_line = Span::from("Last Modified: ").style(INFO_ITEM_HEADER_STYLE)
        + Span::from(last_modified_human_readable);

    let description = table
        .table_info
        .table_description
        .clone()
        .unwrap_or_default();
    name_text
        + Line::from("")
        + warning_text
        + Text::from(path_line)
        + game_name_line
        + rom_line
        + b2s_line
        + last_modified_line
        + Line::from("")
        + Text::from(description)
}

impl From<&IndexedTable> for ListItem<'_> {
    fn from(table: &IndexedTable) -> Self {
        let file_stem = table
            .path
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let line = Some(table.table_info.table_name.to_owned())
            .filter(|s| !s.clone().unwrap_or_default().is_empty())
            .map(|s| {
                Span::from(capitalize_first_letter(s.unwrap_or_default().as_str()))
                    + Span::from(" ")
                    + Span::from(file_stem.clone()).add_modifier(Modifier::DIM)
            })
            .unwrap_or(Line::from(file_stem));
        ListItem::new(line)
    }
}

// fn dialog(app: &mut State, f: &mut Frame) {
//     let dialog_rect = centered_rect(f.area(), 50, 50);
//     f.render_widget(Clear, dialog_rect);
//     f.render_widget(
//         Paragraph::new(format!(
//             "
//         Press `Esc`, `Ctrl-C` or `q` to stop running.\n\
//         Press `j` and `k` to increment and decrement the counter respectively.\n\
//         Counter: {}
//       ",
//             app.counter
//         ))
//         .block(
//             Block::default()
//                 .title("Counter App")
//                 .title_alignment(Alignment::Center)
//                 .borders(Borders::ALL)
//                 .border_type(BorderType::Rounded),
//         )
//         .style(Style::default().fg(Color::Yellow))
//         .alignment(Alignment::Center),
//         dialog_rect,
//     )
// }

// fn centered_rect(r: Rect, percent_x: u16, percent_y: u16) -> Rect {
//     let popup_layout = Layout::default()
//         .direction(Direction::Vertical)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_y) / 2),
//                 Constraint::Percentage(percent_y),
//                 Constraint::Percentage((100 - percent_y) / 2),
//             ]
//             .as_ref(),
//         )
//         .split(r);
//
//     Layout::default()
//         .direction(Direction::Horizontal)
//         .constraints(
//             [
//                 Constraint::Percentage((100 - percent_x) / 2),
//                 Constraint::Percentage(percent_x),
//                 Constraint::Percentage((100 - percent_x) / 2),
//             ]
//             .as_ref(),
//         )
//         .split(popup_layout[1])[1]
// }
