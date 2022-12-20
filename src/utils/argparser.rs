use clap::Parser;

#[derive(Parser, Default, Debug)]
#[clap(name = "HND Node", version, about = "Hash Delivery Network node, written in Rust")]
pub struct ArgsParser {
    /// Address of host to listen to
    #[arg(long, value_name("HOST"), default_value="0.0.0.0")]
    pub host: String,

    /// Port to listen to
    #[arg(short, long, value_name("PORT"), default_value_t=7777)]
    pub port: u16,

    /// Student name (it's being sent each time when someone connects)
    #[arg(long, value_name("NAME"), default_value="shufl9dka")]
    pub student: String,
}
