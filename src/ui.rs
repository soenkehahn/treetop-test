use crate::{process::Process, tree::Forest, R};
use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    widgets::{Paragraph, Widget},
};
use sysinfo::{ProcessRefreshKind, System, UpdateKind};

pub(crate) fn run_ui(tree: Forest<Process>, system: System) -> R<()> {
    app::run_ui(PorcApp::new(tree, system))
}

struct PorcApp {
    tree: Forest<Process>,
    pattern: String,
    system: System,
}

impl PorcApp {
    fn new(tree: Forest<Process>, system: System) -> Self {
        PorcApp {
            tree,
            pattern: "".to_string(),
            system,
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
        Paragraph::new(
            self.tree
                .format(|p| p.name.contains(&self.pattern), area.width),
        )
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

    fn tick(&mut self) {
        self.system.refresh_processes_specifics(
            ProcessRefreshKind::new()
                .with_memory()
                .with_cpu()
                .with_exe(UpdateKind::OnlyIfNotSet),
        );
        self.tree = Process::new_from_sysinfo(
            self.system
                .processes()
                .values()
                .filter(|process| process.thread_kind().is_none()),
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
    use std::{io::stdout, time::Instant};
    use std::{io::Stdout, time::Duration};

    pub(crate) trait App {
        fn tick(&mut self);

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
        std::panic::set_hook(Box::new(|panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            eprintln!("panic: {}", panic_info);
        }));
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        terminal.clear()?;
        let tick_length = Duration::from_millis(1000);
        let mut last_tick = Instant::now();
        app.tick();
        redraw(&mut terminal, &app)?;
        loop {
            let has_event = event::poll(
                tick_length
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_default(),
            )?;
            if has_event {
                let event = event::read()?;
                if let event::Event::Key(key) = event {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                break
                            }
                            code if key.modifiers.is_empty() => {
                                app.update(code);
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                app.tick();
                last_tick = Instant::now();
            }
            redraw(&mut terminal, &app)?;
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
