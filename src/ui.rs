use std::default;
use std::io::{self, stdout, Stdout};

use crossterm::event::KeyCode;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::Frame;
use ratatui::widgets::{block::*,*};
use crossterm::{event, execute, terminal::*};

use crate::machine::ProgramState;
use crate::interface::{UiInterface,ProgramStep,RegisterState};

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

#[derive(Debug,Default)]
pub struct MainUiState {
    prog_states:Vec<ProgramStep>,
    exit:bool
}

impl MainUiState {
    pub fn main_loop(&mut self, terminal:&mut Tui, input:&mut impl UiInterface) -> io::Result<()> {
        while !self.exit {
            let latest_steps = input.get_steps();
            
            
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
        let default_state = ProgramStep::default();
        let current_state = self.prog_states.last().unwrap_or(&default_state);

        frame.render_widget(&current_state.registers, root_layout[0]);
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
}

impl Widget for &RegisterState {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {
        let text = vec![
            format!("R0:{} R1:{} R2:{} R3:{}  PC:{}",self.registers[0],self.registers[1],self.registers[2],self.registers[3],self.program_counter).into(),
            format!("R4:{} R5:{} R6:{} R7:{}  ST:{}",self.registers[4],self.registers[5],self.registers[6],self.registers[7],self.stack_depth).into()
        ];
        let par = Paragraph::new(text).block(Block::default().title("registers").borders(Borders::ALL).border_set(border::THICK));
        par.render(area,buf)
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
