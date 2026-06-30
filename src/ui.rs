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

#[derive(Debug)]
pub struct UI {
    audio_device: MixerDeviceSink,
    instrument: Instrument,
    last_key: char,
    last_freq: String,
    exit: bool,
}

const BLACK_KEYS: &[char] = &['2', '3', '4', ' ', '6', '7', ' ', '9', '0', '-'];
const WHITE_KEYS: &[char] = &['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']'];
const ALL_KEYS: &[char] = &[
    '2', '3', '4', '6', '7', '9', '0', '-', 'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[',
    ']',
];

impl UI {
    pub fn new(audio_device: MixerDeviceSink) -> Self {
        UI {
            audio_device: audio_device,
            instrument: Instrument::new(Sine),
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
        // let _text = format!("Output config: {:?}", audio_device.config());
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

        let text = Text::from(vec![key_text, black_key_render, white_key_render]);
        Paragraph::new(text)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
