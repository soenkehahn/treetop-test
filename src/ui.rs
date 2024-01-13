use crate::{process::Process, tree::Tree, R};
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use sysinfo::Pid;

pub(crate) fn run_ui(tree: Tree<Pid, Process>) -> R<()> {
    app::run_ui(PorcApp::new(tree))
}

struct PorcApp {
    tree: Tree<Pid, Process>,
    pattern: String,
}

impl PorcApp {
    fn new(tree: Tree<Pid, Process>) -> Self {
        PorcApp {
            tree,
            pattern: "".to_string(),
        }
    }
}

impl app::App for PorcApp {
    fn update(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(key) if key.is_ascii() => {
                self.pattern.push(key);
            }
            KeyCode::Backspace => {
                self.pattern.pop();
            }
            _ => {}
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.tree.format(|p| p.name.contains(&self.pattern)))
            .white()
            .on_black()
            .render(
                Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: area.height - 1,
                },
                buf,
            );
        Paragraph::new(format!("search pattern: {}", self.pattern))
            .black()
            .on_white()
            .render(
                Rect {
                    x: area.x,
                    y: area.height - 1,
                    width: area.width,
                    height: 1,
                },
                buf,
            );
    }
}

mod app {
    use crate::R;
    use crossterm::{
        event::{self, KeyCode, KeyEventKind, KeyModifiers},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    };
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        prelude::{CrosstermBackend, Terminal},
        widgets::Widget,
    };
    use std::io::stdout;
    use std::io::Stdout;

    pub(crate) trait App {
        fn update(&mut self, key: KeyCode);

        fn render(&self, area: Rect, buf: &mut Buffer);
    }

    struct AppWrapper<T>(T);

    impl<T: App> Widget for &mut AppWrapper<&T> {
        fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
            self.0.render(area, buf);
        }
    }

    pub(crate) fn run_ui<T: App>(mut app: T) -> R<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        std::panic::set_hook(Box::new(|_panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
        }));
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;
        redraw(&mut terminal, &app)?;

        loop {
            if event::poll(std::time::Duration::from_millis(250))? {
                let event = event::read()?;
                if let event::Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break
                            }
                            code => {
                                app.update(code);
                            }
                        }
                    }
                }
                redraw(&mut terminal, &app)?;
            }
        }

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn redraw<T: App>(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &T) -> R<()> {
        terminal.draw(|frame| {
            frame.render_widget(&mut AppWrapper(app), frame.size());
        })?;
        Ok(())
    }
}
