use crate::midi::MIDI_BUFFER;
use crate::utils::{A440, get_freq_for_note, get_mag_spectrum};
use crate::{ENGINE, log};
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::supports_keyboard_enhancement,
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
use std::{
    cell::RefCell,
    format,
    sync::{Arc, Mutex},
    vec,
};
use std::{io::Result, sync::atomic::Ordering, time::Duration};
use synth_core::{
    SAMPLE_RATE,
    engines::fm::FMSynth,
    midi::{MIDI_NOTE_FREQS, MidiMessage},
    parameter::{
        self,
        ParameterChange::{Decrement, Increment},
        UserParameters,
    },
    voices::Voice,
};

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

/// Maps a computer keyboard key to its corresponding MIDI note
fn get_midi_note_for_key(code: KeyCode) -> Option<u8> {
    match code {
        KeyCode::Char('q') => Some(53), // F
        KeyCode::Char('2') => Some(54),
        KeyCode::Char('w') => Some(55), // G
        KeyCode::Char('3') => Some(56),
        KeyCode::Char('e') => Some(57), // A
        KeyCode::Char('4') => Some(58),
        KeyCode::Char('r') => Some(59), // B
        KeyCode::Char('t') => Some(60), // Middle C
        KeyCode::Char('6') => Some(61),
        KeyCode::Char('y') => Some(62), // D
        KeyCode::Char('7') => Some(63),
        KeyCode::Char('u') => Some(64), // E
        KeyCode::Char('i') => Some(65), // F
        KeyCode::Char('9') => Some(66),
        KeyCode::Char('o') => Some(67), // G
        KeyCode::Char('0') => Some(68),
        KeyCode::Char('p') => Some(69), // A
        KeyCode::Char('-') => Some(70),
        KeyCode::Char('[') => Some(71), // B
        KeyCode::Char(']') => Some(72), // C
        _ => None,
    }
}

/// Sends the computer keyboard events to `MIDI_BUFFER`
fn send_midi(message: MidiMessage) {
    match MIDI_BUFFER.try_send(message) {
        Ok(()) => {}
        Err(_) => log::push("MIDI buffer full"),
    };
}

#[derive(Debug)]
pub struct UI {
    engine: &'static Mutex<RefCell<FMSynth>>,
    last_key: char,
    last_freq: String,
    controls_interface: Controls,
    key_release_reported: bool,
    held_note: Option<u8>,
    exit: bool,
}

impl UI {
    pub fn new(engine: &'static Mutex<RefCell<FMSynth>>) -> Self {
        UI {
            engine,
            controls_interface: Controls::new(0),
            last_key: '_',
            last_freq: String::from("_"),
            key_release_reported: false,
            held_note: None,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        )?;

        self.key_release_reported = supports_keyboard_enhancement().unwrap_or(false);
        if !self.key_release_reported {
            log::push("Terminal does not report key releases; keyboard is monophonic");
        }

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

        if let Some(midi_note) = get_midi_note_for_key(key_event.code) {
            if let Some(pressed_key) = key_event.code.as_char() {
                let freq = MIDI_NOTE_FREQS[midi_note as usize];
                self.last_key = pressed_key;
                self.last_freq = format!("{freq}");
            }

            // Without release events, release the previous note before starting this one
            if !self.key_release_reported {
                if let Some(held_note) = self.held_note.replace(midi_note) {
                    send_midi(MidiMessage(128, held_note, 0));
                }
            }

            send_midi(MidiMessage(144, midi_note, 100));
        }

        if self.controls_interface.is_editing {
            let param_index = self.controls_interface.param_focus_index;

            if self.controls_interface.focus_index == 0 {
                let e = self.engine.lock().unwrap();
                let mut engine = e.borrow_mut();
                let params = engine.get_parameters();

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
                        engine.update_parameter(param_index, Increment);
                    }
                    KeyCode::Down => {
                        engine.update_parameter(param_index, Decrement);
                    }
                    _ => (),
                }
            } else {
                /*let effect = &mut self.instrument.effects[self.controls_interface.focus_index - 1];
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
                }*/
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
        if let Some(midi_note) = get_midi_note_for_key(key_event.code) {
            if self.held_note == Some(midi_note) {
                self.held_note = None;
            }
            send_midi(MidiMessage(128, midi_note, 0));
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn render_keyboard(&self, area: Rect, buf: &mut Buffer) {
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
            .render(area, buf);
    }

    /*fn render_controls(&self, area: Rect, buf: &mut Buffer) {
        let control_subsections = Layout::vertical([Constraint::Length(7), Constraint::Fill(1)])
            .spacing(1)
            .split(area);

        let render_control_cell = |focus_i: usize,
                                   name: &str,
                                   parameters: Vec<Parameter>,
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

        let instrument_cell = Layout::horizontal([Constraint::Max(80), Constraint::Fill(1)])
            .split(control_subsections[0])[0];
        render_control_cell(
            0,
            "FM",
            self.instrument.get_parameters(),
            instrument_cell,
            buf,
        );

        let effect_rows_layout = Layout::vertical(
            (0..self.controls_interface.effect_row_count).map(|_| Constraint::Length(7)),
        )
        .spacing(1);
        let effect_cols_layout = Layout::horizontal(
            (0..self.controls_interface.effect_column_count).map(|_| Constraint::Length(24)),
        );
        let effect_cells: Vec<Rect> = effect_rows_layout
            .split(control_subsections[1])
            .iter()
            .flat_map(|&row| effect_cols_layout.split(row).to_vec())
            .collect();

        /*for (j, cell) in effect_cells.iter().enumerate() {
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
        }*/
    }*/

    /*fn render_visualizations(&self, area: Rect, buf: &mut Buffer) {
        let visualization_subsections =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)])
                .spacing(1)
                .split(area);

        let sample_history = self.instrument.get_sample_history();
        let sample_history = sample_history.lock().unwrap();

        // waveform
        let index_sample_pairs: Vec<(f64, f64)> = sample_history
            .iter()
            .enumerate()
            .map(|(i, v)| (i as f64, *v as f64))
            .collect();

        let waveform_dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Red))
            .data(&index_sample_pairs);

        Chart::new(vec![waveform_dataset])
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .bounds([0.0, SAMPLE_HISTORY_SIZE as f64])
                    .style(Style::default().fg(Color::Gray)),
            )
            .y_axis(
                Axis::default()
                    .bounds([-1.0, 1.0])
                    .style(Style::default().fg(Color::Gray)),
            )
            .render(visualization_subsections[0], buf);

        // magnitude spectrum (logarithmic x-axis)
        // TODO: add x-axis labels (non-trivial to do nice octave-spaced labels in ratatui; leaving for now)
        let min_freq = SAMPLE_RATE as f64 / SAMPLE_HISTORY_SIZE as f64;
        let max_freq = SAMPLE_RATE as f64 / 2.0;

        let samples = sample_history.iter().map(|s| *s).collect();
        let freq_mag_pairs: Vec<(f64, f64)> = get_mag_spectrum(&samples)
            .into_iter()
            .filter(|(freq, _)| *freq >= min_freq)
            .map(|(freq, mag)| (freq.log10(), mag))
            .collect();
        let mag_dataset = Dataset::default()
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Yellow))
            .data(&freq_mag_pairs);

        Chart::new(vec![mag_dataset])
            .block(Block::bordered())
            .x_axis(
                Axis::default()
                    .bounds([min_freq.log10(), max_freq.log10()])
                    .style(Style::default().fg(Color::Gray))
                    .title("freq (Hz)"),
            )
            .y_axis(
                Axis::default()
                    .bounds([0.0, 30.0])
                    .style(Style::default().fg(Color::Gray))
                    .title("magnitude"),
            )
            .render(visualization_subsections[1], buf);
    }*/

    fn render_log_panel(&self, area: Rect, buf: &mut Buffer) {
        let visible_rows = area.height.saturating_sub(2) as usize;
        let log_lines: Vec<Line> = log::snapshot()
            .iter()
            .rev()
            .take(visible_rows)
            .rev()
            .map(|line| Line::from(line.clone()))
            .collect();
        Paragraph::new(log_lines)
            .block(Block::bordered().title(" Log "))
            .render(area, buf);
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

        container.render(area, buf);
        self.render_keyboard(sections[0], buf);
        // self.render_controls(sections[1], buf);
        // self.render_visualizations(sections[2], buf);
        self.render_log_panel(log_area, buf);
    }
}
