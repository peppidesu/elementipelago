//! Recipe graph generation

use bevy::platform::collections::HashMap;

struct RNG {
    seed_x: u64,
    seed_y: u64,
}

impl RNG {
    pub fn init(seed: u64) -> Self {
        RNG {
            seed_x: seed,
            seed_y: seed << 1,
        }
    }

    pub fn get_random(&mut self) -> u64 {
        let mut x = self.seed_x;
        let y = self.seed_y;
        self.seed_x = self.seed_y;
        x ^= x << 23;
        x ^= x >> 17;
        x ^= y;
        self.seed_y = x.wrapping_add(y);
        x
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Status {
    INPUT,
    INTERMEDIATE,
    OUTPUT,
}

pub fn create_graph(
    inputs: u64,
    outputs: u64,
    seed: u64,
    intermediates: u64,
    start_items: u64,
) -> (HashMap<(u64, u64), u64>, Vec<Status>) {
    let mut recipes = HashMap::new();
    let items_len = inputs + intermediates + outputs;
    let mut statuses = vec![Status::INTERMEDIATE; items_len as usize];

    let mut rng = RNG::init(seed);
    for idx in 0..(start_items as usize) {
        statuses[idx] = Status::INPUT;
    }

    for i in start_items..items_len {
        let mut item1: u64;
        let mut item2: u64;
        loop {
            item1 = rng.get_random() % i;
            item2 = rng.get_random() % i;
            if !recipes.contains_key(&(item1, item2)) && !recipes.contains_key(&(item2, item1)) {
                break;
            }
        }
        recipes.insert((item1, item2), i);
    }

    let r_items_len = items_len - start_items;
    let mut inputs_to_place = inputs - start_items;
    while inputs_to_place > 0 {
        let idx = ((rng.get_random() % r_items_len) + start_items) as usize;
        if statuses[idx] != Status::INTERMEDIATE {
            continue;
        }
        statuses[idx] = Status::INPUT;
        inputs_to_place -= 1;
    }

    let mut outputs_to_place = inputs - start_items;
    while outputs_to_place > 0 {
        let idx = ((rng.get_random() % r_items_len) + start_items) as usize;
        if statuses[idx] != Status::INTERMEDIATE {
            continue;
        }
        statuses[idx] = Status::OUTPUT;
        outputs_to_place -= 1;
    }

    (recipes, statuses)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rng_values() {
        let mut rng = RNG::init(29992);
        for i in 0..100 {
            println!("value {} -> {}", i, rng.get_random());
        }
        assert_eq!(true, false)
    }

    #[test]
    fn test_graph() {
        println!("graph {:?}", create_graph(10, 10, 2827108, 10, 4));
        assert_eq!(true, false)
    }
}
