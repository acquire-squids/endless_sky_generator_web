#![allow(dead_code)]
// it's random with a W because I got the algorithm from Wikipedia
pub mod shuffle_index {
    use super::XoShiRo256SS;

    pub trait ShuffleIndex {
        fn len(&self) -> usize;

        fn shuffled_indices_with_rng(&self, rng: &mut XoShiRo256SS) -> Vec<usize> {
            let mut indices = (0..(self.len())).collect::<Vec<usize>>();

            for i in (1..(self.len())).rev() {
                let j = rng.rand_range(0, (i as u64) + 1);
                indices.swap(
                    usize::try_from(j).expect("The index swap range will always be within a usize"),
                    i,
                );
            }

            indices
        }

        fn shuffled_indices(&self, seed: u64) -> Vec<usize> {
            let mut rng = XoShiRo256SS::new(seed);
            self.shuffled_indices_with_rng(&mut rng)
        }
    }

    impl<T> ShuffleIndex for &[T] {
        fn len(&self) -> usize {
            <[T]>::len(self)
        }
    }

    impl<T> ShuffleIndex for Vec<T> {
        fn len(&self) -> usize {
            self.len()
        }
    }

    impl<const N: usize, T> ShuffleIndex for [T; N] {
        fn len(&self) -> usize {
            self.as_slice().len()
        }
    }
}

pub mod weighted_choice {
    use super::XoShiRo256SS;

    pub trait WeightedChoice<T> {
        fn weight_at(&self, index: usize) -> Option<u64>;

        fn item_at(&self, index: usize) -> Option<&T>;

        fn choose(&self, seed: u64) -> Option<&T> {
            let mut rng = XoShiRo256SS::new(seed);
            self.choose_with_rng(&mut rng)
        }

        fn choose_with_rng(&self, rng: &mut XoShiRo256SS) -> Option<&T> {
            use std::collections::BTreeMap;

            let mut btree = BTreeMap::<u64, Vec<&T>>::new();
            let mut total_weight = 0;

            for i in 0.. {
                match (self.weight_at(i), self.item_at(i)) {
                    (Some(weight), Some(item)) => {
                        total_weight += weight;

                        btree
                            .entry(weight)
                            .and_modify(|items| {
                                items.push(item);
                            })
                            .or_insert_with(|| vec![item]);
                    }
                    (_, _) => break,
                }
            }

            let mut pick = rng.rand_range(0, total_weight);

            for (weight, items) in &btree {
                for item in items {
                    match pick.checked_sub(*weight) {
                        Some(0) | None => return Some(*item),
                        Some(next) => pick = next,
                    }
                }
            }

            btree.pop_last().and_then(|(_, mut items)| items.pop())
        }
    }

    impl<T, U> WeightedChoice<T> for &[(T, U)]
    where
        U: Clone + Into<u64>,
    {
        fn weight_at(&self, index: usize) -> Option<u64> {
            self.get(index).map(|(_, weight)| weight.clone().into())
        }

        fn item_at(&self, index: usize) -> Option<&T> {
            self.get(index).map(|(item, _)| item)
        }
    }

    impl<T, U> WeightedChoice<T> for Vec<(T, U)>
    where
        U: Clone + Into<u64>,
    {
        fn weight_at(&self, index: usize) -> Option<u64> {
            self.get(index).map(|(_, weight)| weight.clone().into())
        }

        fn item_at(&self, index: usize) -> Option<&T> {
            self.get(index).map(|(item, _)| item)
        }
    }

    impl<const N: usize, T, U> WeightedChoice<T> for [(T, U); N]
    where
        U: Clone + Into<u64>,
    {
        fn weight_at(&self, index: usize) -> Option<u64> {
            self.get(index).map(|(_, weight)| weight.clone().into())
        }

        fn item_at(&self, index: usize) -> Option<&T> {
            self.get(index).map(|(item, _)| item)
        }
    }
}

pub struct XoShiRo256SS {
    state: [u64; 4],
}

impl XoShiRo256SS {
    pub const fn new(seed: u64) -> Self {
        let mut splitmix = SplitMix64::new(seed);
        let mut state = [0; 4];

        state[0] = splitmix.step();
        state[1] = splitmix.step();
        state[2] = splitmix.step();
        state[3] = splitmix.step();

        Self { state }
    }

    pub const fn step(&mut self) -> u64 {
        let value = self.state[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);

        let t = self.state[1].wrapping_shl(17);

        self.state[2] ^= self.state[0];
        self.state[3] ^= self.state[1];
        self.state[1] ^= self.state[2];
        self.state[0] ^= self.state[3];

        self.state[2] ^= t;
        self.state[3] = self.state[3].rotate_left(45);

        value
    }

    pub fn rand_range(&mut self, minimum: u64, maximum: u64) -> u64 {
        maximum
            .max(minimum)
            .checked_sub(minimum.min(maximum))
            .map_or(0, |num_range| {
                if num_range == 0 {
                    return num_range;
                }

                let bits = num_range.checked_ilog2().unwrap_or_default() + 1;

                if bits < 64 {
                    let mut num = self.step() % 1u64.wrapping_shl(bits);

                    while num >= num_range {
                        num = self.step() % 1u64.wrapping_shl(bits);
                    }

                    num + minimum
                } else {
                    self.step()
                }
            })
    }
}

struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    const fn step(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);

        let value = (self.state ^ self.state.wrapping_shr(30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);

        let value = (value ^ value.wrapping_shr(27)).wrapping_mul(0x94D0_49BB_1331_11EB);

        value ^ value.wrapping_shr(31)
    }
}
