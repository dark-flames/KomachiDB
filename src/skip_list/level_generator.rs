use rand::random;

pub trait LevelGenerator {
    fn max_level(&self) -> usize;

    fn generate_level(&self) -> usize;
}

pub struct RandomLevelGenerator {
    max_level: usize,
    p: f32,
}

#[allow(dead_code)]
impl RandomLevelGenerator {
    pub fn new(max_level: usize, p: f32) -> RandomLevelGenerator {
        RandomLevelGenerator { max_level, p }
    }
}

impl LevelGenerator for RandomLevelGenerator {
    fn max_level(&self) -> usize {
        self.max_level
    }

    fn generate_level(&self) -> usize {
        let mut level = 0;

        while level < self.max_level() {
            if random::<f32>() <= self.p {
                level += 1;
            } else {
                break;
            }
        }

        level
    }
}
