mod room_map;

use room_map::*;
use std::sync::{Arc, Mutex};
use std::thread::{spawn, JoinHandle, available_parallelism};
use std::sync::mpsc::channel;
use std::time::Duration;

fn main() {
    println!("hello world! Let's find the route to that treasure!");

    let (main_sender,main_receiver) = channel::<VecRoute>();
    let shared_receiver = Arc::new(Mutex::new(main_receiver));
    let core_count:usize = available_parallelism().expect("Could not determine the number of cores.").into();
    let mut thread_pool:Vec<JoinHandle<()>> = Vec::with_capacity(core_count);
    let results= Arc::new(Mutex::new(Vec::new()));
    main_sender.send(VecRoute::new()).expect("Could not send initial route.");

    for _ in 0..core_count {
        let thread_sender = main_sender.clone();
        let thread_receiver = shared_receiver.clone();
        let result_clone = results.clone();
        thread_pool.push(spawn(move || {
            loop {
                let new_item = {
                    thread_receiver.lock().expect("Could not lock.").recv_timeout(Duration::from_secs(1))
                };
                let new_item = match new_item {
                    Ok(start) => start,
                    Err(_) => break,
                };
                if !new_item.verify() {
                    continue
                }
                let (start_x, start_y) = match new_item.coordinates() {
                    Some((x,y)) => (x,y),
                    None => continue,
                };
                if start_x == 0 && start_y == 0 && new_item.len() > 0 {
                    continue
                }
                else if start_x == 3 && start_y == 3 && new_item.orb_weight() == 30 {
                    result_clone.lock().expect("Could not lock results.").push(new_item);
                } else {
                    thread_sender.send(new_item.add_step(StepDirection::North)).expect("Could not send new North route.");
                    thread_sender.send(new_item.add_step(StepDirection::East)).expect("Could not send new East route.");
                    thread_sender.send(new_item.add_step(StepDirection::South)).expect("Could not send new South route.");
                    thread_sender.send(new_item.add_step(StepDirection::West)).expect("Could not send new West route.");

                }
            }
        }))
    }
    
    for handle in thread_pool.into_iter() {
        handle.join().expect("Something went wrong joining the thread.");
    }

    let mut finished = results.lock().expect("Could not lock results").clone();
    finished.sort_by(|a, b| a.len().cmp(&b.len()));
    println!("Shortest route: {}",finished.last().expect("Zero routes found, somehow."));
    println!("Longest route: {}",finished.first().expect("Zero routes found, somehow."));
}