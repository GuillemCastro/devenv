use clap::Clap;

#[derive(Debug)]
#[derive(Clap)]
#[clap(version = "0.1", author = "Guillem Castro <guillemcastro4@gmail.com>")]
pub struct Options {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Debug)]
#[derive(Clap)]
pub enum SubCommand {
    Init(Init)
}

#[derive(Debug)]
#[derive(Clap)]
pub struct Init {

}