use std::io::{self, stdout, Stdout};

use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::Frame;
use ratatui::widgets::{block::*,*};
use crossterm::{event, execute, terminal::*};

use crate::machine::ProgramState;

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
    prog_states:Vec<ProgramState>,
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
        let root_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Length(4),Constraint::Fill(1),Constraint::Length(3)])
            .split(frame.size());
        let mid_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Fill(1),Constraint::Length(16)])
            .split(root_layout[1]);
        let default_state = ProgramState::default();
        let current_state = self.prog_states.last().unwrap_or(&default_state);

        frame.render_widget(self.get_register_header(current_state), root_layout[0]);
        frame.render_widget(Paragraph::new("Terminal goes here.").block(Block::default().title("Terminal").borders(Borders::ALL).border_set(border::THICK)),mid_layout[0]);
        frame.render_widget(Paragraph::new("Executed instructions go here").block(Block::default().title("Instructions").borders(Borders::ALL).border_set(border::THICK)), mid_layout[1]);
        frame.render_widget(self, root_layout[2]);
    }

    fn handle_input(&mut self) -> io::Result<()> {
        match event::read()? {
            event::Event::Key(k) if k.code == KeyCode::Esc => {
                //Quick hack
                self.exit = true;
            },
            _ => {}
        }
        Ok(())
    }

    fn get_register_header(&self, state:&ProgramState) -> Paragraph {
        let text = vec![
        format!("R0:{} R1:{} R2:{} R3:{}  PC:{}",state.registers[0],state.registers[1],state.registers[2],state.registers[3],state.program_counter).into(),
        format!("R4:{} R5:{} R6:{} R7:{}  ST:{}",state.registers[4],state.registers[5],state.registers[6],state.registers[7],state.stack_depth).into()];
        Paragraph::new(text).block(Block::default().title("registers").borders(Borders::ALL).border_set(border::THICK))
    }

}

impl Widget for &MainUiState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
            //Set up the layout.

            let footer = Title::from(Line::from(vec![
                "press ".into(),
                "space".bold().blue(),
                " to start/stop the VM".into()
            ]));
            let block = Block::default()
                .title(footer.alignment(Alignment::Center).position(Position::Bottom))
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .render(area,buf);
    }
}
