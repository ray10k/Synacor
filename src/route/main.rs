mod room_map;

use room_map::*;

fn main() {
    println!("hello world! Let's find the route to that treasure!");
    let mut unchecked:Vec<Route> = Vec::with_capacity(4096);
    let mut best = Route::worst_route();
    unchecked.push(Route::new());
    while !unchecked.is_empty() {
        let mut start = unchecked.pop().expect("Somehow, the unchecked queue was both not empty and could not be popped from.");
        if !start.verify() {
            continue;//route is *definitely* not viable, go do something else.
        }
        let (start_x,start_y) = start.coordinates();
        println!("from {start_x},{start_y}.");
        for _ in start_y..=3 {
            unchecked.push(start.add_step(StepDirection::West).unwrap_or(Route::worst_route()));
            unchecked.push(start.add_step(StepDirection::South).unwrap_or(Route::worst_route()));
            unchecked.push(start.add_step(StepDirection::East).unwrap_or(Route::worst_route()));
            if let Some(next_step) = start.add_step(StepDirection::North) {
                start = next_step;
            }
        }
        for _ in start_x..=3 {
            unchecked.push(start.add_step(StepDirection::North).unwrap_or(Route::worst_route()));
            unchecked.push(start.add_step(StepDirection::West).unwrap_or(Route::worst_route()));
            unchecked.push(start.add_step(StepDirection::South).unwrap_or(Route::worst_route()));
            if let Some(next_step) = start.add_step(StepDirection::East) {
                start = next_step;
            }
        }
        //At this point, the orb should be at (3,3), the exit.
        let final_weight = start.orb_weight();
        //if the weight is exactly 30, this is a valid route.
        if final_weight == 30 && start.step_count < best.step_count {
            best = start;
        }
    }
    print!("{best}");
}