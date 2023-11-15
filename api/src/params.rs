use compact_str::CompactString;
use serde::Serialize;
use std::collections::HashMap;

pub type Params = HashMap<CompactString, serde_json::Value>;

pub trait ToParams {
    fn to_params(&self) -> eyre::Result<Params>;
}

impl<T: Serialize> ToParams for T {
    fn to_params(&self) -> eyre::Result<Params> {
        Ok(serde_json::from_str::<Params>(&serde_json::to_string(
            self,
        )?)?)
    }
}
