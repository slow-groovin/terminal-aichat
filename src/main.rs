use tokio::runtime::Runtime;

mod config;
mod cli;
mod chat;


fn main()->Result<(),Box<dyn std::error::Error>>{
    let rt = Runtime::new().unwrap();
    rt.block_on(cli::cli::main())
}