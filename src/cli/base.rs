use crate::cli::arg::Args;

#[derive(Clone, Default)]
pub struct CliBase {
    pub args: Args,
}

pub type Cli = CliBase;