use rayon::prelude::*;

fn main() {
    println!("Trying to find a needle in a 0x7fff haystack!");
    let results: Vec<u16> = (1u16..=0x7fff)
        .into_par_iter()
        .filter(|constant| round_two(4, 1, *constant) == 6)
        .collect();
    println!("Found the following results: {results:?}");
}

fn round_one(a: u16, b: u16, c: u16) -> u16 {
    if a == 0 {
        return b + 1;
    }
    if b != 0 {
        let a = a - 1;
        let b = round_one(a, b, c);
        return round_one(a, b, c);
    }
    return round_one(a - 1, c, c);
}

fn round_two(a:u16,b:u16,c:u16) -> u16 {
    if a > 0 {
        if b > 0 {
            let new_b = b -1;
            let new_a = round_two(a,new_b,c);
            let new_b = new_a;
            let new_a = a - 1;
            return round_two(new_a,new_b,c);
        }
        let new_a = a - 1;
        let new_b = c;
        return round_two(new_a,new_b,c);
    }
    return b + 1;
}