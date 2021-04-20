use noise::{Seedable, NoiseFn, OpenSimplex};

pub struct NoiseGenerator {
    generator: OpenSimplex,
}

impl NoiseGenerator {
    pub fn from_seed(seed: u32) -> Self {
        let generator = OpenSimplex::new();
        generator.set_seed(seed);

        NoiseGenerator { generator }
    }

    pub fn get(&self, x: f64, z: f64) -> f64{
        self.generator.get([x, z])
    }
}