use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CurrentScreen};
use nes::cpu::{CpuFlags, CPU};

pub fn register_format(name: String, value: String) -> Vec<Span<'static>> {
    vec![
        Span::styled(name + ": ", Style::default().fg(Color::Green)),
        Span::styled(value, Style::default().fg(Color::Yellow)), // Variable with different color
    ]
}

fn flag_string(flag: CpuFlags) -> &'static str {
    match flag {
        CpuFlags::CARRY => "C",
        CpuFlags::ZERO => "Z",
        CpuFlags::INTERRUPT_DISABLE => "I",
        CpuFlags::DECIMAL_MODE => "D",
        CpuFlags::BREAK => "B",
        CpuFlags::BREAK2 => "-", // Often unused or implementation-specific
        CpuFlags::OVERFLOW => "V",
        CpuFlags::NEGATIVE => "N",
        _ => "UNKNOWN",
    }
}

pub fn flag_format(flag: CpuFlags, cpu: &CPU) -> Vec<Span<'_>> {
    let mut color: Color = Color::Red;
    if cpu.flags.contains(flag) {
        color = Color::Green
    }
    vec![
        Span::styled(" ", Style::default().fg(color)),
        Span::styled(flag_string(flag), Style::default().fg(color)),
    ]
}

pub fn ui(frame: &mut Frame, app: &App, cpu: &CPU) {
    // Create the layout sections.
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Current Memory Page
            Constraint::Min(1),    // Memory Addresses
            Constraint::Length(5), // Registers
            Constraint::Length(2), // Flags
            Constraint::Length(1), // Instructions, 2 to allow for gap on top
            Constraint::Length(1), // Terminal/input area
        ])
        .split(frame.area());

    // TITLE
    let title_block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    // Create a formatted string for your variable
    let page_value = format!("{:02X}", app.current_page); // Hex format example

    // Create a collection of differently styled spans
    let title_spans = vec![
        Span::styled("Current Memory Page: ", Style::default().fg(Color::Green)),
        Span::styled(page_value, Style::default().fg(Color::Yellow)), // Variable with different color
    ];

    // Create a paragraph from the collection of spans
    let title = Paragraph::new(Line::from(title_spans)).block(title_block);

    frame.render_widget(title, chunks[0]); // Renders the terminal

    // Memory Addresses
// Create a single Line with multiple Spans for horizontal display
let mut spans = Vec::new();
for i in 0..16 { // Increased from 9 to 16 for a better display
    // Add each memory value as a span
    spans.push(Span::styled(format!("{:02X} ", cpu.memory[i]), Style::default().fg(Color::Yellow)));
    
    // Optional: Add a separator every 4 bytes for readability
    if (i + 1) % 8 == 0 && i < 15 {
        spans.push(Span::styled(" | ", Style::default().fg(Color::Gray)));
    }
}

let memory_line = Line::from(spans);
let memory_para = Paragraph::new(vec![memory_line]);

frame.render_widget(memory_para, chunks[1]);


    // Registers area

    // Create registers for cpu
    let cpu_a = format!("0x{:02X}", cpu.a); // Hex format example
    let cpu_x = format!("0x{:02X}", cpu.x);
    let cpu_y = format!("0x{:02X}", cpu.y);
    let cpu_pc = format!("0x{:02X}", cpu.pc);
    let cpu_sp = format!("0x{:02X}", cpu.sp);
    // TODO Add last instruction

    let a_span = register_format("A".to_string(), cpu_a);
    let x_span = register_format("X".to_string(), cpu_x);
    let y_span = register_format("Y".to_string(), cpu_y);

    let pc_span = register_format("PC".to_string(), cpu_pc);
    let sp_span = register_format("SP".to_string(), cpu_sp);
    // TODO add last instruction

    let register_fh_span = vec![Line::from(a_span), Line::from(x_span), Line::from(y_span)];
    let register_first_half =
        Paragraph::new(register_fh_span).block(Block::default().borders(Borders::ALL));

    let register_sh_span = vec![Line::from(pc_span), Line::from(sp_span)];
    let register_second_half =
        Paragraph::new(register_sh_span).block(Block::default().borders(Borders::ALL));

    let footer_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[2]);


    frame.render_widget(register_first_half, footer_chunks[0]);
    frame.render_widget(register_second_half, footer_chunks[1]);

    let mut all_flag_spans: Vec<Span> = Vec::new();

    // Add the "Flags: " span
    all_flag_spans.push(Span::styled("Flags:", Style::default().fg(Color::White)));

    // Add spans from each flag_format call
    for flag in &[
        CpuFlags::NEGATIVE,
        CpuFlags::OVERFLOW,
        CpuFlags::BREAK2,
        CpuFlags::BREAK,
        CpuFlags::DECIMAL_MODE,
        CpuFlags::INTERRUPT_DISABLE,
        CpuFlags::ZERO,
        CpuFlags::CARRY,
    ] {
        all_flag_spans.extend(flag_format(*flag, cpu));
    }
    let flag_para = Paragraph::new(Line::from(all_flag_spans));
    frame.render_widget(flag_para, chunks[3]);

    // Instructions
    let inst_para = Paragraph::new(Span::styled("Press q to quit.", Style::default().fg(Color::White)));
    frame.render_widget(inst_para, chunks[4]);

    // Input area
    let terminal_para = Paragraph::new(app.terminal_input.clone());
    frame.render_widget(terminal_para, chunks[5]);

    // Exiting screen
    if let CurrentScreen::Exiting = app.current_screen {
        frame.render_widget(Clear, frame.area()); //this clears the entire screen and anything already drawn
        let popup_block = Block::default()
            .title("Y/N")
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let exit_text = Text::styled("Would you like to quit?", Style::default().fg(Color::Red));
        // the `trim: false` will stop the text from being cut off when over the edge of the block
        let exit_paragraph = Paragraph::new(exit_text)
            .block(popup_block)
            .wrap(Wrap { trim: false });

        let area = centered_rect(60, 25, frame.area());
        frame.render_widget(exit_paragraph, area);
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
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
