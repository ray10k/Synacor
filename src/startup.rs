use std::thread;
use std::io;

use crate::ui::{MainUiState,start_ui,stop_ui};

use crate::machine::VirtualMachine;
use crate::thread_interface::make_interfaces;


pub(crate) fn main_interface(mut loaded_data:VirtualMachine)->io::Result<()>{
    let mut term = start_ui()?;
    let (mut ui_interface, vm_interface) = make_interfaces();
    let mut user_interface = MainUiState::new();

    {
        thread::spawn( move || {
            //VM thread
            let mut vm_interface = vm_interface;
            loaded_data.run_program(&mut vm_interface);
        });
        user_interface.main_loop(&mut term, &mut ui_interface)?;
    }

    stop_ui()?;
    Ok(())
}
