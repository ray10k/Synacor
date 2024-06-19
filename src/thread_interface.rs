use std::sync::{mpsc::{self,Sender,Receiver},atomic::{AtomicBool,Ordering},Arc};
use std::io::{Error,ErrorKind,Result as IoResult};

use crate::{UiInterface,VmInterface,RuntimeState,ProgramStep};

pub fn make_interfaces() -> (ThreadUiInterface,ThreadVmInterface) {
    let (state_out,state_in) = mpsc::channel();
    let (input_out,input_in) = mpsc::channel();
    let (output_out,output_in) = mpsc::channel();
    let (steps_out,steps_in) = mpsc::channel();
    let need_input = Arc::new(AtomicBool::new(false));

    let ui_inter = ThreadUiInterface{
        need_input : need_input.clone(),
        state_outgoing : state_out,
        input_outgoing : input_out,
        output_incoming : output_in,
        steps_incoming : steps_in
    };
    let vm_inter = ThreadVmInterface{
        need_input : need_input.clone(),
        state_incoming : state_in,
        input_incoming : input_in,
        output_outgoing : output_out,
        steps_outgoing : steps_out,
    };
    (ui_inter,vm_inter)
}


pub struct ThreadUiInterface {
    /* tbd */
    need_input:Arc<AtomicBool>,
    state_outgoing:Sender<RuntimeState>,
    input_outgoing:Sender<String>,
    output_incoming:Receiver<char>,
    steps_incoming:Receiver<ProgramStep>,
}

pub struct ThreadVmInterface {
    /* tbd */
    need_input: Arc<AtomicBool>,
    state_incoming:Receiver<RuntimeState>,
    input_incoming:Receiver<String>,
    output_outgoing:Sender<char>,
    steps_outgoing:Sender<ProgramStep>,
}

unsafe impl Send for ThreadUiInterface {}
unsafe impl Send for ThreadVmInterface {}

impl UiInterface for ThreadUiInterface {
    fn read_output(&mut self) -> Option<String> {
        let out = self.output_incoming.try_iter();
        let buffer = String::from_iter(out);
        if buffer.len() > 0 {
            Some(buffer)
        } else {
            None
        }
    }

    fn read_steps(&mut self) -> Vec<ProgramStep> {
        Vec::from_iter(self.steps_incoming.try_iter())
    }

    fn need_input(&self) -> bool {
        self.need_input.load(Ordering::Relaxed)
    }

    fn is_finished(&self) -> bool {
        todo!()
    }

    fn write_input(&mut self, input:&str) -> IoResult<()> {
        let res = self.input_outgoing.send(String::from(input));
        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorKind::Other, "Could not send input")),
        }
    }

    fn write_state(&mut self, input:RuntimeState) -> std::io::Result<()> {
        let res = self.state_outgoing.send(input);
        match res {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::new(ErrorKind::Other, "Could not send state")),
        }
    }
}

impl VmInterface for ThreadVmInterface {
    fn write_output(&mut self, c:char) -> std::io::Result<()> {
        match self.output_outgoing.send(c){
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }

    fn write_step(&mut self, step:ProgramStep) -> std::io::Result<()> {
        if step.instruction == "IN" {
            self.need_input.swap(true, Ordering::Relaxed);
        }
        match self.steps_outgoing.send(step){
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
        }
    }

    fn runtime_err(&mut self, s:String) {
        //Throwing this into the void for now.
        drop(s);
    }

    fn read_input(&mut self) -> String {
        let input = self.input_incoming.recv();
        match input {
            Ok(s) => s,
            Err(_) => String::from(""),
        }
    }

    fn read_state(&mut self, blocking:bool) -> Option<RuntimeState> {
        if blocking {
            match self.state_incoming.recv() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        } else {
            //not blocking.
            match self.state_incoming.try_recv() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
        
    }
}