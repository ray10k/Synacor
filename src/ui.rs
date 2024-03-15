use std::io::{self, stdout, Stdout};

use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::Frame;
use ratatui::widgets::{block::*,*};
use crossterm::{execute,terminal::*};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn start_ui() -> io::Result<Tui> {
    execute!(stdout(),EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn stop_ui() -> io::Result<()> {
    execute!(stdout(),LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub trait UiIo {
    fn get_output(&mut self) -> Option<String>;
    fn send_input(&mut self, input:&str) -> io::Result<()>;
}

#[derive(Debug,Default)]
pub struct MainUiState {
    registers:[u16;8],
    prog_count:usize,
    stack_count:usize,
    recent_instructions:Vec<String>,
    terminal_io:Vec<String>,
    exit:bool
}

impl MainUiState {
    pub fn main_loop(&mut self, terminal:&mut Tui /* , io:impl UiIo*/) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_input()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame:&mut Frame){
        frame.render_widget(self, frame.size())
    }

    fn handle_input(&mut self) -> io::Result<()> {
        todo!();
    }
}

impl Widget for &MainUiState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
            let title = Title::from("Synacor challenge VM".bold());
            let footer = Title::from(Line::from(vec![
                "press ".into(),
                "space".bold().blue(),
                " to start/stop the VM".into()
            ]));
            let block = Block::default()
                .title(title.alignment(Alignment::Center))
                .title(footer.alignment(Alignment::Center).position(Position::Bottom))
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .render(area,buf);
    }
}
