mod cln;
mod utils;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::str::FromStr;
use utils::to_cln;

/// cln-grpc playground
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// eg. http://127.0.0.1
    #[arg(short, long, env, default_value_t = String::from_str("http://127.0.0.1").unwrap())]
    url: String,

    /// eg. 10010
    #[arg(short, long, env, default_value_t = String::from_str("10010").unwrap())]
    nport: String,

    /// eg /home/sphinx
    #[arg(long, env, default_value_t = String::from_str(".").unwrap())]
    creds: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// getinfo
    Getinfo,
    /// keysend
    Keysend {
        /// eg 5
        #[arg(short, long, env, default_value_t = 5u64)]
        amount_sat: u64,

        /// eg 02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0
        #[arg(short, long, env, default_value_t = String::from_str("02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0").unwrap())]
        destination: String,

        /// eg 02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0
        #[arg(short, long, env, default_value_t = String::from_str("02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0").unwrap())]
        lhpk: String,

        /// eg 5917632481235
        #[arg(short, long, env, default_value_t = 5917632481235)]
        scid: u64,

        /// eg 1000
        #[arg(short, long, env, default_value_t = 1000)]
        base: u64,

        /// eg 40
        #[arg(short, long, env, default_value_t = 40)]
        expirydelta: u32,

        /// eg 1
        #[arg(short, long, env, default_value_t = 1)]
        prop: u32,
    },
    Getroute {
        /// eg 5
        #[arg(short, long, env, default_value_t = 5u64)]
        amount_sat: u64,

        /// eg 02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0
        #[arg(short, long, env, default_value_t = String::from_str("02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0").unwrap())]
        destination: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    dotenv().ok();
    let args = Args::parse();

    // global arguments
    let url = args.url;
    let port = args.nport;
    let command = args.command;
    let creds_dir = args.creds;

    let creds = utils::collect_creds(&creds_dir).await?;
    let mut client = cln::ClnRPC::try_new(&url, &port, &creds, 50).await?;

    match command {
        Commands::Getinfo => {
            let info = client.get_info().await?;
            println!("INFO {:?}", info);
        }
        Commands::Keysend {
            destination,
            amount_sat,
            lhpk,
            scid,
            base,
            expirydelta,
            prop,
        } => {
            let amt_msat = amount_sat * 1000;
            let scid = to_cln(scid);
            let hm = client
                .keysend_with_route_hint(
                    &destination,
                    amt_msat,
                    &lhpk,
                    &scid,
                    base,
                    expirydelta,
                    prop,
                )
                .await?;
            println!("HM {:?}", hm);
        }
        Commands::Getroute {
            destination,
            amount_sat,
        } => {
            let amt_msat = amount_sat * 1000;
            let hm = client.get_route(&destination, amt_msat).await?;
            println!("HM: {:?}", hm);
        }
    }
    Ok(())
}
