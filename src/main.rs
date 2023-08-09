mod cln;
mod utils;

use clap::Parser;
use std::str::FromStr;

/// cln-grpc playground
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// eg. http://127.0.0.1
    #[arg(short, long, default_value_t = String::from_str("http://127.0.0.1").unwrap())]
    url: String,

    /// eg. 10010
    #[arg(short, long, default_value_t = String::from_str("10010").unwrap())]
    nport: String,

    /// eg info
    #[arg(short, long, default_value_t = String::from_str("info").unwrap())]
    command: String,

    /// eg 5
    #[arg(short, long, default_value_t = 5u64)]
    amount_sat: u64,

    /// eg 02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0
    #[arg(short, long, default_value_t = String::from_str("02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0").unwrap())]
    destination: String,

    /// eg 02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0
    #[arg(short, long, default_value_t = String::from_str("02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0").unwrap())]
    lhpk: String,

    /// eg 5917632481235
    #[arg(short, long, default_value_t = 5917632481235)]
    scid: u64,

    /// eg 1000
    #[arg(short, long, default_value_t = 1000)]
    base: u64,

    /// eg 40
    #[arg(short, long, default_value_t = 40)]
    expirydelta: u32,

    /// eg 1
    #[arg(short, long, default_value_t = 1)]
    prop: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = Args::parse();
    let url = args.url;
    let port = args.nport;
    let command = args.command;
    let pk = args.destination;
    let amt_msat = args.amount_sat * 1000;
    let lhpk = args.lhpk;
    let scid = to_cln(args.scid);
    let feebase = args.base;
    let expirydelta = args.expirydelta;
    let feeprop = args.prop;

    let creds_dir = ".";
    let creds = utils::collect_creds(creds_dir).await?;
    let mut client = cln::ClnRPC::try_new(&url, &port, &creds, 50).await?;

    if &command == "info" {
        let info = client.get_info().await?;
        println!("INFO {:?}", info);
    } else if &command == "keysend" {
        let hm = client
            .keysend_with_route_hint(&pk, amt_msat, &lhpk, &scid, feebase, expirydelta, feeprop)
            .await?;
        println!("HM {:?}", hm);
    }
    Ok(())
}

fn to_cln(scid: u64) -> String {
    let block = scid >> 40 & 0xffffff;
    let tx = scid >> 16 & 0xffffff;
    let output = scid & 0xffff;
    format!("{}x{}x{}", block, tx, output)
}
