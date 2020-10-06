pub trait LevelGenerator {
    fn max_level(&self) -> usize;

    fn generate_level(&self) -> usize;
}
