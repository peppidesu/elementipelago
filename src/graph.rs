//! Recipe graph generation

use bevy::platform::collections::{HashMap, HashSet};

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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Status {
    INPUT,
    INTERMEDIATE,
    OUTPUT,
}

pub type Element = (u64, Status);

pub fn create_graph(
    inputs: u64,
    outputs: u64,
    seed: u64,
    intermediates: u64,
    start_items: u64,
) -> (HashMap<(Element, Element), Vec<Element>>, Vec<Element>) {
    let mut dag_edges: Vec<(usize, usize, u64, Status)> = Vec::new();
    let mut used: HashSet<(usize, usize)> = HashSet::new();
    let mut rng = RNG::init(seed);

    let mut inputs_to_place: Vec<u64> = (1..=inputs).collect();
    let mut intermediates_to_place: Vec<u64> = (1..=intermediates).collect();
    let mut outputs_to_place: Vec<u64> = (1..=outputs).collect();

    for i in 1..=start_items {
        dag_edges.push((0, 0, i, Status::INPUT));
        inputs_to_place.retain(|&x| x != i);
    }

    let mut inputs_placed = 0;
    let mut outputs_placed = 0;

    let mut to_place_length =
        inputs_to_place.len() + intermediates_to_place.len() + outputs_to_place.len();
    while to_place_length > 0 {
        let previous_items = dag_edges.len();
        let mut new_layer = Vec::new();
        let max_layer_size =
            usize::min(previous_items * previous_items / 2 - 1, to_place_length - 1) as u64;
        let new_layer_size = if max_layer_size == 0 {
            1
        } else {
            (rng.get_random() % max_layer_size) + 1
        };

        for _ in 0..new_layer_size {
            let mut to_place_type = None;
            while to_place_type.is_none() {
                let typ = rng.get_random() % 3;
                if typ == 0 && outputs_placed > inputs_placed && inputs_to_place.len() > 0 {
                    to_place_type = Some(Status::INPUT);
                    inputs_placed += 1;
                } else if typ == 1 && intermediates_to_place.len() > 0 {
                    to_place_type = Some(Status::INTERMEDIATE);
                } else if typ == 2 && outputs_to_place.len() > 0 {
                    to_place_type = Some(Status::OUTPUT);
                    outputs_placed += 1;
                }
            }

            let to_place_type = to_place_type.unwrap();

            let mut input1_idx = rng.get_random() as usize % previous_items;
            let mut input2_idx = rng.get_random() as usize % previous_items;

            while used.contains(&(input1_idx, input2_idx)) {
                input1_idx = rng.get_random() as usize % previous_items;
                input2_idx = rng.get_random() as usize % previous_items;
            }

            used.insert((input1_idx, input2_idx));
            used.insert((input2_idx, input1_idx));

            let output_idx = (rng.get_random()
                % match to_place_type {
                    Status::INPUT => inputs_to_place.len(),
                    Status::INTERMEDIATE => intermediates_to_place.len(),
                    Status::OUTPUT => outputs_to_place.len(),
                } as u64) as usize;

            let output = match to_place_type {
                Status::INPUT => inputs_to_place.remove(output_idx),
                Status::INTERMEDIATE => intermediates_to_place.remove(output_idx),
                Status::OUTPUT => outputs_to_place.remove(output_idx),
            };

            new_layer.push((input1_idx, input2_idx, output, to_place_type));
        }

        to_place_length =
            inputs_to_place.len() + intermediates_to_place.len() + outputs_to_place.len();
        dag_edges.extend(new_layer);
    }

    // TODO: we may want to build the recipes in the hashmap directly eventually
    let mut recipes_with_outputs: HashMap<(Element, Element), Vec<Element>> = HashMap::new();
    for (in1, in2, output, typ) in dag_edges.iter() {
        let input1;
        let input2;
        if in1 < in2 {
            input1 = *in1;
            input2 = *in2;
        } else {
            input1 = *in2;
            input2 = *in1;
        }
        let to_insert = match *typ {
            Status::INPUT => (((0, Status::INPUT), (0, Status::INPUT)), (*output, *typ)),
            Status::INTERMEDIATE | Status::OUTPUT => {
                let i1 = dag_edges[input1];
                let i2 = dag_edges[input2];
                (((i1.2, i1.3), (i2.2, i2.3)), (*output, *typ))
            }
        };

        if recipes_with_outputs.contains_key(&to_insert.0) {
            recipes_with_outputs
                .get_mut(&to_insert.0)
                .expect("key is still in hashmap after check")
                .push(to_insert.1);
        } else {
            recipes_with_outputs.insert(to_insert.0, vec![to_insert.1]);
        }
    }

    (
        recipes_with_outputs,
        (1..=inputs)
            .map(|x| (x, Status::INPUT))
            .chain((1..=intermediates).map(|x| (x, Status::INTERMEDIATE)))
            .chain((1..=outputs).map(|x| (x, Status::OUTPUT)))
            .collect(),
    )
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
