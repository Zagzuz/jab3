pub trait Persistence {
    type Input;
    type Output;

    fn serialize(&self) -> eyre::Result<Self::Output>;

    fn deserialize(&mut self, input: Self::Input) -> eyre::Result<()>;
}
