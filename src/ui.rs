use crate::fm_synth_instrument::{FMSynthInstrument, SAMPLE_HISTORY_SIZE};
use crate::log;
use crate::parameter::ParameterChange::{Decrement, Increment};
use crate::parameter::UserParameters;
use crate::utils::{A440, get_freq_for_note};
use crate::voices::Voice;
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
};
use ratatui::symbols;
use ratatui::widgets::{Axis, Chart, Dataset, GraphType};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Flex, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use std::format;
use std::{io::Result, sync::atomic::Ordering, time::Duration};

const TICK_RATE: Duration = Duration::from_millis(50);

const BLACK_KEYS: &[char] = &['2', '3', '4', ' ', '6', '7', ' ', '9', '0', '-'];
const WHITE_KEYS: &[char] = &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']'];
const ALL_KEYS: &[char] = &[
    '2', '3', '4', '6', '7', '9', '0', '-', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[',
    ']',
];

#[derive(Debug)]
struct Controls {
    effect_row_count: usize,
    effect_column_count: usize,
    effect_count: usize,
    // focus_index: 0 -> instrument, 1..=effect_count -> effects
    focus_index: usize,
    is_editing: bool,
    param_focus_index: usize,
}

impl Controls {
    pub fn new(effect_count: usize) -> Self {
        let effect_column_count = effect_count.min(3).max(1);
        let effect_row_count = effect_count.div_ceil(effect_column_count);
        Controls {
            effect_row_count,
            effect_column_count,
            effect_count,
            focus_index: 0,
            is_editing: false,
            param_focus_index: 0,
        }
    }

    pub fn select(&mut self) {
        self.param_focus_index = 0;
        self.is_editing = true;
    }

    pub fn deselect(&mut self) {
        self.param_focus_index = 0;
        self.is_editing = false;
    }

    pub fn move_right(&mut self) {
        if self.is_editing || self.focus_index == 0 {
            return;
        }
        let eff_idx = self.focus_index - 1;
        if eff_idx % self.effect_column_count == self.effect_column_count - 1 {
            self.focus_index -= self.effect_column_count - 1;
        } else {
            self.focus_index += 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.is_editing || self.focus_index == 0 {
            return;
        }
        let eff_idx = self.focus_index - 1;
        if eff_idx % self.effect_column_count == 0 {
            self.focus_index += self.effect_column_count - 1;
        } else {
            self.focus_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.is_editing {
            return;
        }
        if self.focus_index == 0 {
            if self.effect_count > 0 {
                self.focus_index = 1;
            }
        } else {
            let eff_idx = self.focus_index - 1;
            self.focus_index = 1 + (eff_idx + self.effect_column_count) % self.effect_count;
        }
    }

    pub fn move_up(&mut self) {
        if self.is_editing || self.focus_index == 0 {
            return;
        }
        let eff_idx = self.focus_index - 1;
        if eff_idx < self.effect_column_count {
            self.focus_index = 0;
        } else {
            self.focus_index -= self.effect_column_count;
        }
    }
}

#[derive(Debug)]
pub struct UI {
    instrument: FMSynthInstrument,
    last_key: char,
    last_freq: String,
    controls_interface: Controls,
    exit: bool,
}

impl UI {
    pub fn new(instrument: FMSynthInstrument) -> Self {
        UI {
            controls_interface: Controls::new(instrument.effects.len()),
            instrument,
            last_key: '_',
            last_freq: String::from("_"),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        )?;

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        execute!(std::io::stdout(), PopKeyboardEnhancementFlags)?;
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self) -> Result<()> {
        if !event::poll(TICK_RATE)? {
            return Ok(());
        }

        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_press(key_event)
            }
            Event::Key(key_event) if key_event.kind == KeyEventKind::Release => {
                self.handle_key_release(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_press(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Char('c')
            && key_event.modifiers.contains(KeyModifiers::CONTROL)
        {
            self.exit();
        }

        let freq = match key_event.code {
            KeyCode::Char('q') => get_freq_for_note(-4), // F
            KeyCode::Char('2') => get_freq_for_note(-3),
            KeyCode::Char('w') => get_freq_for_note(-2), // G
            KeyCode::Char('3') => get_freq_for_note(-1),
            KeyCode::Char('e') => A440, // A
            KeyCode::Char('4') => get_freq_for_note(1),
            KeyCode::Char('r') => get_freq_for_note(2), // B
            KeyCode::Char('t') => get_freq_for_note(3), // C
            KeyCode::Char('6') => get_freq_for_note(4),
            KeyCode::Char('y') => get_freq_for_note(5), // D
            KeyCode::Char('7') => get_freq_for_note(6),
            KeyCode::Char('u') => get_freq_for_note(7), // E
            KeyCode::Char('i') => get_freq_for_note(8), // F
            KeyCode::Char('9') => get_freq_for_note(9),
            KeyCode::Char('o') => get_freq_for_note(10), // G
            KeyCode::Char('0') => get_freq_for_note(11),
            KeyCode::Char('p') => get_freq_for_note(12), // A
            KeyCode::Char('-') => get_freq_for_note(13),
            KeyCode::Char('[') => get_freq_for_note(14), // B
            KeyCode::Char(']') => get_freq_for_note(15), // C
            _ => 0.0,
        };

        if freq > 0.0 {
            if let Some(pressed_key) = key_event.code.as_char() {
                self.last_key = pressed_key;
                self.last_freq = format!("{freq}");
                let mut voice = self
                    .instrument
                    .voices
                    .voice_on(pressed_key as u8)
                    .lock()
                    .unwrap();
                voice.set_freq(freq);
                voice.on.store(true, Ordering::Relaxed);
            }
        }

        if self.controls_interface.is_editing {
            let param_index = self.controls_interface.param_focus_index;

            if self.controls_interface.focus_index == 0 {
                let params = self.instrument.get_parameters();

                match key_event.code {
                    KeyCode::Esc => self.controls_interface.deselect(),
                    KeyCode::Left => {
                        if param_index > 0 {
                            self.controls_interface.param_focus_index -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if param_index < params.len() - 1 {
                            self.controls_interface.param_focus_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        self.instrument.update_parameter(param_index, Increment);
                    }
                    KeyCode::Down => {
                        self.instrument.update_parameter(param_index, Decrement);
                    }
                    _ => (),
                }
            } else {
                let effect = &mut self.instrument.effects[self.controls_interface.focus_index - 1];
                let params = effect.get_parameters();

                match key_event.code {
                    KeyCode::Esc => self.controls_interface.deselect(),
                    KeyCode::Left => {
                        if param_index > 0 {
                            self.controls_interface.param_focus_index -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if param_index < params.len() - 1 {
                            self.controls_interface.param_focus_index += 1;
                        }
                    }
                    KeyCode::Up => {
                        effect.update_parameter(param_index, Increment);
                    }
                    KeyCode::Down => {
                        effect.update_parameter(param_index, Decrement);
                    }
                    _ => (),
                }
            }
        } else {
            match key_event.code {
                KeyCode::Enter => self.controls_interface.select(),
                KeyCode::Left => self.controls_interface.move_left(),
                KeyCode::Right => self.controls_interface.move_right(),
                KeyCode::Up => self.controls_interface.move_up(),
                KeyCode::Down => self.controls_interface.move_down(),
                _ => (),
            }
        }
    }

    fn handle_key_release(&mut self, key_event: KeyEvent) {
        if let Some(k) = key_event.code.as_char() {
            if ALL_KEYS.contains(&k) {
                if let Some(voice) = self.instrument.voices.voice_off(k as u8) {
                    voice.lock().unwrap().on.store(false, Ordering::Relaxed);
                }
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &UI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" Terminal Synth ".bold());
        let instructions = Line::from(vec![
            " Navigate ".into(),
            "<↑/↓/←/→> ".bold(),
            " Select Section ".into(),
            "<Enter> ".bold(),
            " Select Param ".into(),
            "<←/→> ".bold(),
            " Update Param ".into(),
            "<↑/↓> ".bold(),
            " Deselect Section ".into(),
            "<Esc> ".bold(),
            " Quit ".into(),
            "<Ctrl+C> ".red().bold(),
        ]);
        let container = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.right_aligned())
            .border_set(border::THICK);

        let inner_area = container.inner(area);
        container.render(area, buf);

        let main_sections =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(8)]).split(inner_area);
        let main_area = main_sections[0];
        let log_area = main_sections[1];

        let outer_columns = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .flex(Flex::SpaceBetween);

        let sections = outer_columns.split(main_area);

        // render keyboard
        let key_text = Line::from(vec![
            "Key: ".into(),
            self.last_key.to_string().green(),
            " Freq: ".into(),
            self.last_freq.clone().blue(),
        ]);
        let black_key_render = Line::from(
            std::iter::once(Span::raw("  "))
                .chain(BLACK_KEYS.iter().flat_map(|key| {
                    [
                        Span::styled("|", Style::default().fg(Color::White).bg(Color::Black)),
                        Span::styled(
                            format!(" {key} "),
                            Style::default()
                                .fg(Color::White)
                                .bg(if *key == self.last_key {
                                    Color::Blue
                                } else {
                                    Color::Black
                                }),
                        ),
                    ]
                }))
                .chain(std::iter::once(Span::styled(
                    "|",
                    Style::default().fg(Color::White).bg(Color::Black),
                )))
                .collect::<Vec<_>>(),
        );
        let white_key_render = Line::from(
            WHITE_KEYS
                .iter()
                .flat_map(|key| {
                    [
                        Span::styled("|", Style::default().fg(Color::Black).bg(Color::White)),
                        Span::styled(
                            format!(" {key} "),
                            Style::default()
                                .fg(Color::Black)
                                .bg(if *key == self.last_key {
                                    Color::Blue
                                } else {
                                    Color::White
                                }),
                        ),
                    ]
                })
                .chain(std::iter::once(Span::styled(
                    "|",
                    Style::default().fg(Color::Black).bg(Color::White),
                )))
                .collect::<Vec<_>>(),
        );

        let keyboard = Text::from(vec![key_text, black_key_render, white_key_render]);
        Paragraph::new(keyboard)
            .left_aligned()
            .block(Block::bordered())
            .render(sections[0], buf);

        // render controls: instrument row on top, effects grid below
        let right_sections = Layout::vertical([Constraint::Length(7), Constraint::Fill(1)])
            .spacing(1)
            .split(sections[1]);

        let render_control_cell = |focus_i: usize,
                                   name: &str,
                                   parameters: Vec<crate::parameter::Parameter>,
                                   cell: Rect,
                                   buf: &mut Buffer| {
            let container = if focus_i == self.controls_interface.focus_index {
                match self.controls_interface.is_editing {
                    true => Block::bordered().on_green(),
                    false => Block::bordered().on_blue(),
                }
            } else {
                Block::bordered()
            };
            let inner_cell = container.inner(cell);
            container.render(cell, buf);

            let sections =
                Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).split(inner_cell);
            Paragraph::new(name.to_string())
                .bold()
                .centered()
                .render(sections[0], buf);

            let param_cols =
                Layout::horizontal((0..parameters.len()).map(|_| Constraint::Length(9)))
                    .spacing(1)
                    .split(sections[1]);
            for (j, c) in param_cols.iter().enumerate() {
                let param_block = if self.controls_interface.is_editing
                    && focus_i == self.controls_interface.focus_index
                    && j == self.controls_interface.param_focus_index
                {
                    Block::new().on_cyan()
                } else {
                    Block::new()
                };
                Paragraph::new(format!(
                    "{}\n↑\n{}\n↓",
                    parameters[j].name,
                    parameters[j].render_value()
                ))
                .centered()
                .block(param_block)
                .render(*c, buf);
            }
        };

        // instrument row
        let instrument_cell = Layout::horizontal([Constraint::Max(80), Constraint::Fill(1)])
            .split(right_sections[0])[0];
        render_control_cell(
            0,
            "FM",
            self.instrument.get_parameters(),
            instrument_cell,
            buf,
        );

        // effects grid
        let effect_rows_layout = Layout::vertical(
            (0..self.controls_interface.effect_row_count).map(|_| Constraint::Length(7)),
        )
        .spacing(1);
        let effect_cols_layout = Layout::horizontal(
            (0..self.controls_interface.effect_column_count).map(|_| Constraint::Length(24)),
        );
        let effect_cells: Vec<Rect> = effect_rows_layout
            .split(right_sections[1])
            .iter()
            .flat_map(|&row| effect_cols_layout.split(row).to_vec())
            .collect();

        for (j, cell) in effect_cells.iter().enumerate() {
            if j < self.instrument.effects.len() {
                let effect = &self.instrument.effects[j];
                render_control_cell(
                    j + 1,
                    &effect.get_name(),
                    effect.get_parameters(),
                    *cell,
                    buf,
                );
            }
        }

        // visualizations
        let sample_history = self.instrument.get_sample_history();
        let sample_history = sample_history.lock().unwrap();
        let data: Vec<(f64, f64)> = sample_history
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v as f64))
            .collect();

        let dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Red))
            .data(&data);

        Chart::new(vec![dataset])
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, SAMPLE_HISTORY_SIZE as f64]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([-1.0, 1.0]),
            )
            .render(sections[2], buf);

        // render log panel
        let visible_rows = log_area.height.saturating_sub(2) as usize;
        let log_lines: Vec<Line> = log::snapshot()
            .iter()
            .rev()
            .take(visible_rows)
            .rev()
            .map(|line| Line::from(line.clone()))
            .collect();
        Paragraph::new(log_lines)
            .block(Block::bordered().title(" Log "))
            .render(log_area, buf);
    }
}
