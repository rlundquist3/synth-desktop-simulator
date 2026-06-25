use crate::oscillator::{Oscillator, Waveform::Sine};
use crate::utils::{get_freq_for_note, A440};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use rodio::{MixerDeviceSink, Source};
use std::{io::Result, time::Duration};

#[derive(Debug)]
pub struct UI {
    audio_device: MixerDeviceSink,
    oscillator: Oscillator,
    last_key: String,
    exit: bool,
}

impl UI {
    pub fn new(audio_device: MixerDeviceSink) -> Self {
        UI {
            audio_device: audio_device,
            oscillator: Oscillator::new(Sine),
            last_key: String::from("_"),
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
            self.last_key = format!("{freq}");
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
        let instructions = Line::from(vec![
            "click some buttons".into(),
            " Quit ".into(),
            "<Ctrl+C> ".red().bold(),
        ]);
        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        let key_text = Text::from(vec![Line::from(vec![
            "Freq: ".into(),
            self.last_key.clone().blue(),
        ])]);

        Paragraph::new(key_text)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

//   | 2 | 3 | 4 |   | 6 | 7 |   | 9 | 0 | - |
// | q | w | e | r | t | y | u | i | o | p | [ | ] |
