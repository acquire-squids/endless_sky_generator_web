#![allow(dead_code)]
// it's random with a W because I got the algorithm from Wikipedia
pub trait ShuffleIndex {
    fn shuffled_indices(&self, seed: usize) -> Vec<usize>;
}

impl<T> ShuffleIndex for &[T] {
    fn shuffled_indices(&self, seed: usize) -> Vec<usize> {
        let mut rng = XoShiRo256SS::new(seed as u64);

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
        if maximum.checked_add(minimum).is_none() {
            return self.step();
        }

        maximum.checked_sub(minimum).map_or(minimum, |num_range| {
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
