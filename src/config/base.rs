use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ConfigBase {

}

pub type Config = ConfigBase;