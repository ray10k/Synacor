

pub enum RuntimeState {
    Run,
    Pause,
    SingleStep,
    RunForSteps(usize),
    RunUntilAddress(u16),
}

#[derive(Debug, Clone)]
pub struct RegisterState {
    pub registers:[u16;8],
    pub stack_depth:usize,
    pub program_counter:u16,
}

impl Default for RegisterState {
    fn default() -> Self {
        Self { registers: Default::default(), stack_depth: Default::default(), program_counter: Default::default() }
    }
}

#[derive(Debug,Default,Clone)]
pub struct ProgramStep {
    pub registers:RegisterState,
    pub instruction:String,
}

impl ProgramStep {
    pub const fn const_default() -> Self {
        ProgramStep{
            registers: RegisterState{
                registers:[0;8],
                stack_depth: 0,
                program_counter: 0
            },
            instruction : String::new()
        }
    }
}

pub trait UiInterface {
    fn get_output(&mut self) -> Option<String>;
    fn get_steps(&mut self) -> Vec<ProgramStep>;
    fn send_input(&mut self, input:&str) -> std::io::Result<()>;
    fn send_state(&mut self, input:RuntimeState) -> std::io::Result<()>;
}

pub trait VmInterface {
    fn write_output(&mut self, c:char) -> std::io::Result<()>;
    fn write_step(&mut self, instruction:String, registers:RegisterState) -> std::io::Result<()>;
    fn read_input(&mut self) -> char;
    fn read_state(&mut self) -> Option<RuntimeState>;
}

