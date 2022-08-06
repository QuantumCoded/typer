use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::{borrow::Borrow, io::Error, time::Duration};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::Wrap;
use tui::{backend::Backend, widgets::Paragraph, Terminal};

pub fn main<B: Backend>(terminal: &mut Terminal<B>) -> Result<f64, Error> {
    let mut prompt_text = String::from(include_str!("prompt"));
    let mut user_text = Vec::new();

    loop {
        // if an event is read in the next 10ms then read it
        if event::poll(Duration::from_millis(10))? {
            let event = event::read()?;

            match event {
                // add text to buffer
                Event::Key(KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                }) => {
                    // this only works if user_text has spans that are only 1 char long, which it does
                    let color = if Some(ch) == prompt_text.chars().nth(user_text.len()) {
                        Color::White
                    } else {
                        Color::Red
                    };

                    user_text.push(Span::styled(ch.to_string(), Style::default().fg(color)));
                }

                // remove text from buffer if the last char isn't space
                // FIXME: this breaks if the user accidentally presses space mid word
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                }) => {
                    if Some(" ") != user_text.last().map(|ch| -> &str { ch.content.borrow() }) {
                        user_text.pop();
                    }
                }

                // handle CTRL+C to break loop
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                }) => std::process::exit(1),

                _ => {}
            }
        }

        // needs to be in main loop to rerender text AFTER keypresses
        terminal.draw(|f| {
            let size = f.size();
            let mut spans = user_text.clone();
            let mut remaining_prompt = prompt_text.chars().skip(spans.len());

            if let Some(ch) = remaining_prompt.next() {
                spans.push(Span::styled(
                    ch.to_string(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::UNDERLINED),
                ));
            }

            spans.push(Span::styled(
                remaining_prompt.collect::<String>(),
                Style::default().fg(Color::DarkGray),
            ));

            let paragraph = Paragraph::new(Spans(spans)).wrap(Wrap { trim: true });

            // this clone is probably bad because user_text *could* be huge
            f.render_widget(paragraph, size);
        })?;

        // the user has finished the prompt
        if user_text.len() == prompt_text.len() {
            let mut right = 0;
            let total = prompt_text.len();

            while prompt_text.len() > 0 {
                let ch = user_text.pop().map(|ch| ch.content.into_owned());
                if ch == prompt_text.pop().map(|ch| ch.to_string()) {
                    right += 1;
                }
            }

            std::thread::sleep(Duration::from_secs(1));

            return Ok(right as f64 / total as f64 * 100.);
        }
    }
}
