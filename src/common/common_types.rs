/*! Identifier trait
All identifiers should implement [Identifier] to be useable in processing and pipelines.
!*/
use std::str::FromStr;

use schemars::JsonSchema;

use crate::{error::Error, lang::Lang};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(from = "IdentificationSer", into = "IdentificationSer")]
pub struct Identification {
    label: Lang,
    prob: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct IdentificationSer {
    label: String,
    prob: f32,
}

impl From<Identification> for IdentificationSer {
    fn from(i: Identification) -> Self {
        Self {
            label: i.label.to_string(),
            prob: i.prob,
        }
    }
}
impl From<IdentificationSer> for Identification {
    fn from(i: IdentificationSer) -> Self {
        Self {
            label: Lang::from_str(&i.label).unwrap(),
            prob: i.prob,
        }
    }
}

impl Identification {
    pub fn new(label: Lang, prob: f32) -> Self {
        Self { label, prob }
    }
    /// Get a reference to the identification's label.
    pub fn label(&self) -> &Lang {
        &self.label
    }

    /// Get a reference to the identification's prob.
    pub fn prob(&self) -> &f32 {
        &self.prob
    }
}

#[cfg(test)]
mod tests {

    use super::Identification;
}
