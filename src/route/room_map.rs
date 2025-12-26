
#[derive(Clone, Copy)]
pub enum TileContent {
    Plus,
    Minus,
    Asterisk,
    Value(i8)
}

pub fn generate_map() -> [TileContent;16] {
    use TileContent::*; 
    return [
        Value(22),
        Plus,
        Value(4),
        Asterisk,
        Minus,
        Value(4),
        Asterisk,
        Value(8),
        Value(9),
        Minus,
        Value(11),
        Minus,
        Asterisk,
        Value(18),
        Asterisk,
        Value(1)
    ]
}

#[derive(Clone, Copy)]
pub enum StepDirection {
    North,
    East,
    South,
    West,
    None
}

pub struct Route {
    steps:[StepDirection; 15],
    step_count:u8
}

impl Route {
    pub fn new() -> Self {
        Route{
            steps : [StepDirection::None;15],
            step_count : 0
        }
    }

    pub fn verify(&self) -> bool {
        let map = generate_map();
        let mut current_cell:usize = 0;
        let mut current_weight:usize = 22;
        let mut operation:TileContent = TileContent::Value(22);
        //checks 1 and 2: Does the route ever go outside the 4x4 play-area,
        // and does the orb ever go below 1 weight?
        for current_step in self.steps.iter() {
            let x = current_cell / 4;
            let y = current_cell % 4;
            match current_step {
                StepDirection::North => if y < 3 {
                    current_cell += 1;
                } else {
                    return false
                },
                StepDirection::East => if x < 3 {
                    current_cell += 4;
                } else {
                    return false
                },
                StepDirection::South => if y > 0 {
                    current_cell -= 1;
                } else {
                    return false
                },
                StepDirection::West => if x > 0 {
                    current_cell -= 4;
                } else {
                    return false;
                },
                StepDirection::None => break,
            }
            if let TileContent::Value(value) = map[current_cell] {
                match operation {
                    TileContent::Plus => {
                        current_weight += value as usize;
                    },
                    TileContent::Minus => {
                        if current_weight <= value as usize {
                            return false; //Orb would have hit 0 or below.
                        }
                        current_weight -= value as usize;
                    },
                    TileContent::Asterisk => {
                        current_weight *= value as usize;
                    },
                    TileContent::Value(_) => panic!("Two values in a row?"),
                }
            }
            operation = map[current_cell];
        }
        //check 3: is the current route, *plus* the minimum number of
        // steps to reach the exit, 15 or more steps long?
        let x = current_cell / 4;
        let y = current_cell % 4;
        let manhattan_distance = (3 - x) + (3 - y);
        manhattan_distance + (self.step_count as usize) < 16
    }
}