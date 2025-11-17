use polars::prelude::*;
use indexmap::IndexMap;
use serde::Deserialize;

pub type PositionalData = IndexMap<String, DataFrame>;

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct TempData {
    pub X: IndexMap<String, f64>,
    pub Y: IndexMap<String, f64>,
}
