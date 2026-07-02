use crate::effects::ParameterChange::{Decrement, Increment};
use crate::instrument::Instrument;
use crate::oscillator::Waveform::Sine;
use crate::utils::{A440, get_freq_for_note};
use crossterm::{
    event::{
        self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
};
use ratatui::layout::{Constraint, Flex, Layout};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Widget},
};
use rodio::MixerDeviceSink;
use std::{io::Result, sync::atomic::Ordering};

const BLACK_KEYS: &[char] = &['2', '3', '4', ' ', '6', '7', ' ', '9', '0', '-'];
const WHITE_KEYS: &[char] = &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']'];
const ALL_KEYS: &[char] = &[
    '2', '3', '4', '6', '7', '9', '0', '-', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[',
    ']',
];

#[derive(Debug)]
struct EffectSection {
    row_count: usize,
    column_count: usize,
    total_count: usize,
    focus_index: usize,
    is_editing: bool,
    param_focus_index: usize,
}

impl EffectSection {
    pub fn new(row_count: usize, column_count: usize) -> Self {
        EffectSection {
            row_count,
            column_count,
            total_count: row_count * column_count,
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
        if self.is_editing {
            return;
        }

        if self.focus_index % self.column_count == self.column_count - 1 {
            self.focus_index -= self.column_count - 1;
        } else {
            self.focus_index += 1;
        }
    }

    pub fn move_left(&mut self) {
        if self.is_editing {
            return;
        }

        if self.focus_index % self.column_count == 0 {
            self.focus_index += self.column_count - 1;
        } else {
            self.focus_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.is_editing {
            return;
        }

        self.focus_index = (self.focus_index + self.column_count) % self.total_count;
    }

    pub fn move_up(&mut self) {
        if self.is_editing {
            return;
        }

        if self.focus_index < self.column_count {
            self.focus_index += (self.row_count - 1) * self.column_count;
        } else {
            self.focus_index -= self.column_count;
        }
    }
}

#[derive(Debug)]
pub struct UI {
    audio_device: MixerDeviceSink,
    instrument: Instrument,
    last_key: char,
    last_freq: String,
    effect_section: EffectSection,
    exit: bool,
}

impl UI {
    pub fn new(audio_device: MixerDeviceSink) -> Self {
        let instrument = Instrument::new(Sine);
        let effect_count = instrument.effects.len();
        let column_count = effect_count.min(3);
        let row_count = effect_count.div_ceil(column_count);

        UI {
            audio_device: audio_device,
            instrument,
            last_key: '_',
            last_freq: String::from("_"),
            effect_section: EffectSection::new(row_count, column_count),
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        execute!(
            std::io::stdout(),
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
        )?;

        self.audio_device.mixer().add(self.instrument.clone());

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
            KeyCode::Char('t') => get_freq_for_note(4), // C
            KeyCode::Char('6') => get_freq_for_note(5),
            KeyCode::Char('y') => get_freq_for_note(6), // D
            KeyCode::Char('7') => get_freq_for_note(7),
            KeyCode::Char('u') => get_freq_for_note(8), // E
            KeyCode::Char('i') => get_freq_for_note(9), // F
            KeyCode::Char('9') => get_freq_for_note(10),
            KeyCode::Char('o') => get_freq_for_note(11), // G
            KeyCode::Char('0') => get_freq_for_note(12),
            KeyCode::Char('p') => get_freq_for_note(13), // A
            KeyCode::Char('-') => get_freq_for_note(14),
            KeyCode::Char('[') => get_freq_for_note(15), // B
            KeyCode::Char(']') => get_freq_for_note(16), // C
            _ => 0.0,
        };

        if freq > 0.0 {
            if let Some(pressed_key) = key_event.code.as_char() {
                self.last_key = pressed_key;
                self.last_freq = format!("{freq}");
                let voice = self.instrument.voices.voice_on(pressed_key);
                voice.osc.lock().unwrap().set_freq(freq);
                voice.on.store(true, Ordering::Relaxed);
            }
        }

        if self.effect_section.is_editing {
            let effect = &mut self.instrument.effects[self.effect_section.focus_index];
            let params = effect.get_parameters();
            let param_index = self.effect_section.param_focus_index;

            match key_event.code {
                KeyCode::Esc => self.effect_section.deselect(),
                KeyCode::Left => {
                    if param_index > 0 {
                        self.effect_section.param_focus_index -= 1;
                    }
                }
                KeyCode::Right => {
                    if param_index < params.len() - 1 {
                        self.effect_section.param_focus_index += 1;
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
        } else {
            match key_event.code {
                KeyCode::Enter => self.effect_section.select(),
                KeyCode::Left => self.effect_section.move_left(),
                KeyCode::Right => self.effect_section.move_right(),
                KeyCode::Up => self.effect_section.move_up(),
                KeyCode::Down => self.effect_section.move_down(),
                _ => (),
            }
        }
    }

    fn handle_key_release(&mut self, key_event: KeyEvent) {
        if let Some(k) = key_event.code.as_char() {
            if ALL_KEYS.contains(&k) {
                if let Some(voice) = self.instrument.voices.voice_off(k) {
                    voice.on.store(false, Ordering::Relaxed);
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
        let instructions = Line::from(vec![" Quit ".into(), "<Ctrl+C> ".red().bold()]);
        let container = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.right_aligned())
            .border_set(border::THICK);

        let inner_area = container.inner(area);
        container.render(area, buf);

        let outer_columns = Layout::horizontal([Constraint::Length(60), Constraint::Length(90)])
            .flex(Flex::SpaceBetween);

        let inner_rows =
            Layout::vertical((0..self.effect_section.row_count).map(|_| Constraint::Length(7)))
                .spacing(1);
        let inner_columns = Layout::horizontal(
            (0..self.effect_section.column_count).map(|_| Constraint::Length(18)),
        );

        let outer_cells = outer_columns.split(inner_area);

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
            .render(outer_cells[0], buf);

        // render controls
        let control_rows = inner_rows.split(outer_cells[1]);
        let control_cells = control_rows
            .iter()
            .flat_map(|&row| inner_columns.split(row).to_vec());

        for (i, cell) in control_cells.enumerate() {
            let container = if i == self.effect_section.focus_index {
                match self.effect_section.is_editing {
                    true => Block::bordered().on_green(),
                    false => Block::bordered().on_blue(),
                }
            } else {
                Block::bordered()
            };

            if i < self.instrument.effects.len() {
                let effect = &self.instrument.effects[i];
                let parameters = effect.get_parameters();

                let inner_cell = container.inner(cell);
                container.render(cell, buf);

                let effect_row_constraints =
                    Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]);
                let parameter_columns =
                    Layout::horizontal((0..parameters.len()).map(|_| Constraint::Length(9)));

                let effect_rows = effect_row_constraints.split(inner_cell);
                Paragraph::new(effect.get_name())
                    .centered()
                    .render(effect_rows[0], buf);
                let parameter_cells = parameter_columns.split(effect_rows[1]);

                for (j, c) in parameter_cells.iter().enumerate() {
                    let container = if self.effect_section.is_editing
                        && i == self.effect_section.focus_index
                        && j == self.effect_section.param_focus_index
                    {
                        Block::new().on_cyan()
                    } else {
                        Block::new()
                    };

                    Paragraph::new(format!(
                        "{}\n↑\n{}\n↓",
                        parameters[i].name,
                        parameters[i].get_value()
                    ))
                    .block(container)
                    .render(*c, buf);
                }
            } else {
                Paragraph::new(format!("Effect {:02}", i + 1))
                    .centered()
                    .block(container)
                    .render(cell, buf);
            }
        }
    }
}
