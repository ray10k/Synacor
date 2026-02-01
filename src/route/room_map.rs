use std::fmt::Display;


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
    West
}

impl Display for StepDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", 
            match self {
                StepDirection::North => "north",
                StepDirection::East => "east",
                StepDirection::South => "south",
                StepDirection::West => "west"
            }
        )
    }
}

#[derive(Clone)]
pub struct VecRoute(Vec<StepDirection>);

impl VecRoute {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_step(&self, step:StepDirection) -> Self{
        let mut retval = self.clone();
        retval.0.push(step);
        retval
    }

    pub fn coordinates(&self) -> Option<(i8,i8)> {
        let mut x = 0;
        let mut y = 0;
        for step in self.0.iter() {
            match step {
                StepDirection::North => y += 1,
                StepDirection::East => x += 1,
                StepDirection::South => y -= 1,
                StepDirection::West => x -= 1
            }
            if x < 0 || x > 3 || y < 0 || y > 3 {
                return None // if the path ever goes outside the playfield, 
            } //the route has no determinable end-location since it is invalid.
        }
        Some((x,y))
    }

    pub fn orb_weight(&self) -> usize {
        let mut retval = 22;
        let mut x = 0;
        let mut y = 0;
        let mut last_op = TileContent::Value(22);
        let map = generate_map();

        for step in self.0.iter() {
            match step {
                StepDirection::North => {
                    if y == 3 {
                        return 0;
                    }
                    y += 1},
                StepDirection::East => {
                    if x == 3 {
                        return 0;
                    }
                    x += 1},
                StepDirection::South => {
                    if y == 0 {
                        return 0;
                    }
                    y -= 1;
                },
                StepDirection::West => {
                    if x == 0 {
                        return 0;
                    }
                    x -= 1}
            }
            let index = y + (x * 4);
            let curr_op = map[index];
            if let TileContent::Value(value) = curr_op {
                match last_op {
                    TileContent::Plus => retval += value as usize,
                    TileContent::Minus => {
                        if value as usize >= retval {
                            return 0;
                        }
                        retval -= value as usize;
                    },
                    TileContent::Asterisk => retval *= value as usize,
                    TileContent::Value(_) => (), //just do nothing, since this will trigger on the initial 0,0 coordinate.
                }
            }
            last_op = curr_op;
        }
        return retval;
    }

    pub fn verify(&self,max_depth:usize) -> bool {
        let weight = self.orb_weight();
        if weight == 0 {
            return false;
        }
        if let None = self.coordinates() {
            return false;
        }
        if self.0.len() > max_depth { //excessive, but I want to check something.
            return false;
        }
        return true;
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Display for VecRoute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for step in self.0.iter() {
            let _ = write!(f,"{step}\n")?;
        }
        std::fmt::Result::Ok(())
    }
}