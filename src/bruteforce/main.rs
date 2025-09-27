use rayon::{prelude::*, ThreadPool, ThreadPoolBuilder};
use std::collections::HashMap;
fn main() {
    let main_threadpool: ThreadPool = ThreadPoolBuilder::new()
        .stack_size(1024 * 1024)
        .build()
        .expect("Could not build threadpool.");


    println!("Sanity check with c == 1:");
    let test = round_five(2,1,1);
    println!("result: {test} (should be 5)");

    println!("Trying to find a needle in a 0x7fff haystack!"); 
    
    let results: Vec<u16> = main_threadpool.install(|| {
        (1u16..=0x7fff)
            .into_par_iter()
            .map(|count| {println!(">{count:4x}"); count})
            .filter(|constant| {let result = round_five(4, 1, *constant); println!("<{constant:4x}:{result}");result == 6})
            .collect()
    });
/*
    let results: Vec<u16> = (1u16..=0x7fff)
        .into_iter()
        .filter(|constant| {
            println!("{constant:4x}");
            let result = round_five(4, 1, *constant);
            result == 6})
        .collect();*/
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

fn round_two(a: u16, b: u16, c: u16) -> u16 {
    if a == 0 {
        return (b + 1) & 0x7fff;
    }
    if b == 0 {
        return round_two(a - 1, c, c);
    }
    let new_b = round_two(a, b - 1, c);
    return round_two(a - 1, new_b, c);
}

fn round_three(init_a: u16, init_b: u16, c: u16) -> u16 {
    let mut a = init_a;
    let mut b = init_b;
    loop {
        if a == 0 {
            return (b + 1) & 0x7fff;
        }
        if b == 0 {
            a -= 1;
            b = c;
            continue;
        }
        let temp = round_three(a, b - 1, c);
        a -= 1;
        b = temp;
        continue;
    }
}

fn r4_key (a:u16, b:u16) -> u32 {
    (a as u32) | ((b as u32) << 16)
}

fn round_four(a:u16,b:u16,c:u16,mut lookup:HashMap<u32,u16>) -> (u16,HashMap<u32,u16>) {
    let map_key:u32 = r4_key(a, b);
    if let Some(result) = lookup.get(&map_key) {
        return (*result,lookup);
    }

    if a == 0 {
        let result = (b + 1) & 0x7fff;
        let _ = lookup.insert(map_key, result);
        return (result,lookup);
    }
    if b == 0 {
        let a = a - 1;
        let b = c;
        let (result, mut lookup) = round_four(a,b,c,lookup);
        let map_key = r4_key(a, b);
        let _ = lookup.insert(map_key,result);
        return (result, lookup);
    }
    let b = b - 1;
    let map_key = r4_key(a, b);
    let (b, mut lookup) = round_four(a, b, c, lookup);
    let _ = lookup.insert(map_key, b);
    let a = a - 1;
    let map_key = r4_key(a, b);
    let (result, mut lookup) = round_four(a, b, c, lookup);
    let _ = lookup.insert(map_key, result);
    return (result, lookup);
}

fn round_five(a:u16,b:u16,c:u16) -> u16 {
    let mut stack = Vec::with_capacity(0x1000);
    stack.push(a);
    stack.push(b);
    while stack.len() >= 2 {
        let n = stack.pop(); //a
        let m = stack.pop(); //b
        match (m,n) {
            (Some(0),Some(y)) => {
                stack.push((y+1) & 0x7fff);
            },
            (Some(x),Some(0)) => {
                stack.push(x-1);
                stack.push(c);
            },
            (Some(x),Some(y)) => {
                stack.push(x-1);
                stack.push(x);
                stack.push(y-1);
            },
            (None,_)|(_,None) => {panic!("Remaining Stack too small, somehow")}
        };
    }
    return stack[0];
}


/*

17a1 JT   0007   R0 17a9
17a4 ADD  0009   R0   R1 0001
17a8 RET  0012
     :l17a1
17a9 JT   0007   R1 17b6
17ac ADD  0009   R0   R0 7fff
17b0 SET  0001   R1   R7
17b3 CALL 0011 17a1
17b5 RET  0012
     :l17a9
17b6 PUSH 0002   R0
17b8 ADD  0009   R1   R1 7fff
17bc CALL 0011 17a1
17be SET  0001   R1   R0
17c1 POP  0003   R0
17c3 ADD  0009   R0   R0 7fff
17c7 CALL 0011 17a1
17c9 RET  0012

*/