use crate::oscillator::{Oscillator, Waveform::Sine};
use crate::utils::{A440, get_freq_for_note};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::text::Span;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
};
use rodio::{MixerDeviceSink, Source};
use std::{io::Result, time::Duration};

#[derive(Debug)]
pub struct UI {
    audio_device: MixerDeviceSink,
    oscillator: Oscillator,
    last_key: char,
    last_freq: String,
    exit: bool,
}

impl UI {
    pub fn new(audio_device: MixerDeviceSink) -> Self {
        UI {
            audio_device: audio_device,
            oscillator: Oscillator::new(Sine),
            last_key: '_',
            last_freq: String::from("_"),
            exit: false,
        }
    }

    pub fn set_oscillator(&mut self, oscillator: Oscillator) {
        self.oscillator = oscillator;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    pub fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
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
            self.last_key = match key_event.code.as_char() {
                Some(last_key) => last_key,
                _ => '_',
            };
            self.last_freq = format!("{freq}");
            self.oscillator.set_freq(freq);
            self.audio_device.mixer().add(
                self.oscillator
                    .clone()
                    .take_duration(Duration::from_millis(1000)),
            );
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
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.right_aligned())
            .border_set(border::THICK);

        let key_text = Line::from(vec![
            "Key: ".into(),
            self.last_key.to_string().green(),
            " Freq: ".into(),
            self.last_freq.clone().blue(),
        ]);

        let black_keys = vec!['2', '3', '4', ' ', '6', '7', ' ', '9', '0', '-'];
        let white_keys = vec!['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']'];

        let black_key_render = Line::from(
            std::iter::once(Span::raw("  "))
                .chain(black_keys.iter().flat_map(|key| {
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
            white_keys
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

        let text = Text::from(vec![key_text, black_key_render, white_key_render]);
        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
