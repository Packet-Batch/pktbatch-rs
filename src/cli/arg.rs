use clap::Parser;

#[derive(Parser, Default, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
    #[clap(short='c', long="cfg", default_value = "./config.json")]
    pub config: String,

    #[clap(short='l', long="list")]
    pub list_cfg: bool,

    
}