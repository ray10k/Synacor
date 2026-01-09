mod room_map;

use room_map::*;

fn main() {
    println!("hello world! Let's find the route to that treasure!");
    let mut unchecked:Vec<VecRoute> = Vec::with_capacity(4096);
    let mut finished:Vec<VecRoute> = Vec::new();
    unchecked.push(VecRoute::new());
    while !unchecked.is_empty() {
        let start = unchecked.pop().expect("Somehow, the unchecked queue was both not empty and could not be popped from.");
        if !start.verify() {
            continue;//route is *definitely* not viable, go do something else.
        }
        let (start_x, start_y) = start.coordinates().expect("Valid route somehow went out of range.");
        //println!("from {start_x},{start_y}.");

        if start_x == 3 && start_y == 3 && start.orb_weight() == 30{
            finished.push(start);
        }
        else {
            unchecked.push(start.add_step(StepDirection::North));
            unchecked.push(start.add_step(StepDirection::East));
            unchecked.push(start.add_step(StepDirection::South));
            unchecked.push(start.add_step(StepDirection::West));
        }
    }
    finished.sort_by(|a, b| a.len().cmp(&b.len()));
    println!("Shortest route: {}",finished.last().expect("Zero routes found, somehow."));
    println!("Longest route: {}",finished.first().expect("Zero routes found, somehow."));
}