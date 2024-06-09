
#[derive(PartialEq)]
pub enum RuntimeState {
    Run,
    Pause,
    SingleStep,
    RunForSteps(usize),
    RunUntilAddress(u16),
    Terminate,
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

    pub fn step(registers:RegisterState, instruction:String) -> Self {
        Self { registers: registers, instruction: instruction }
    }
}

pub trait UiInterface {
    fn read_output(&mut self) -> Option<String>;
    fn read_steps(&mut self) -> Vec<ProgramStep>;
    fn need_input(&self) -> bool;
    fn is_finished(&self) -> bool;
    fn write_input(&mut self, input:&str) -> std::io::Result<()>;
    fn write_state(&mut self, input:RuntimeState) -> std::io::Result<()>;
}

pub trait VmInterface {
    fn write_output(&mut self, c:char) -> std::io::Result<()>;
    fn write_step(&mut self, step:ProgramStep) -> std::io::Result<()>;
    fn runtime_err(&mut self, message:String);
    fn read_input(&mut self) -> String;
    fn read_state(&mut self, blocking:bool) -> Option<RuntimeState>;
}

