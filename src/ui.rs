use crate::{process::Process, tree::Tree, R};
use crossterm::event::KeyCode;
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

    fn render(&self) -> String {
        [
            format!("search pattern: {}", self.pattern),
            "".to_string(),
            self.tree.format(|p| p.name.contains(&self.pattern)),
        ]
        .join("\n")
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
        prelude::{CrosstermBackend, Stylize, Terminal},
        widgets::Paragraph,
    };
    use std::io::stdout;
    use std::io::Stdout;

    pub(crate) trait App {
        fn update(&mut self, key: KeyCode);

        fn render(&self) -> String;
    }

    pub(crate) fn run_ui<T: App>(mut app: T) -> R<()> {
        stdout().execute(EnterAlternateScreen)?;
        enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;

        redraw(&mut terminal, &app)?;

        loop {
            if event::poll(std::time::Duration::from_millis(250))? {
                if let event::Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break
                            }
                            code => {
                                app.update(code);
                            }
                        }
                        redraw(&mut terminal, &app)?;
                    }
                }
            }
        }

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    fn redraw<T: App>(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &T) -> R<()> {
        terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new(app.render()).white().on_black(),
                frame.size(),
            );
        })?;
        Ok(())
    }
}
