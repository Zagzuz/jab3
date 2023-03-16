use eyre::ensure;

#[derive(Debug)]
pub struct ImagerConfig {
    pub limit: usize,
}

impl ImagerConfig {
    pub fn verify(&self) -> eyre::Result<()> {
        ensure!(
            matches!(self.limit, 1..=100),
            "amount of results can only be 1-100"
        );
        Ok(())
    }
}

impl Default for ImagerConfig {
    fn default() -> Self {
        Self { limit: 100 }
    }
}
