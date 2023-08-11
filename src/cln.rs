use crate::utils::*;
use anyhow::{anyhow, Result};
use cln_grpc::pb;
use log::info;
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Identity};

pub struct ClnRPC {
    pub client: pb::node_client::NodeClient<Channel>,
}

impl ClnRPC {
    // try new a few times
    pub async fn try_new(url: &str, grpc_port: &str, creds: &Creds, i: usize) -> Result<Self> {
        for iteration in 0..i {
            if let Ok(c) = Self::new(url, grpc_port, creds).await {
                return Ok(c);
            }
            sleep_ms(1000).await;
            info!("retry CLN connect {}", iteration);
        }
        Err(anyhow!("could not connect to CLN"))
    }
    pub async fn new(url: &str, grpc_port: &str, creds: &Creds) -> Result<Self> {
        // println!("CA PEM {:?}", &creds.ca_pem);
        // println!("CLEINT PEM {:?}", &creds.client_pem);
        // println!("CLIENT KEY {:?}", &creds.client_key);

        let ca = Certificate::from_pem(&creds.ca_pem);
        let ident = Identity::from_pem(&creds.client_pem, &creds.client_key);

        let tls = ClientTlsConfig::new()
            .domain_name("cln")
            .identity(ident)
            .ca_certificate(ca);

        let url = format!("{}:{}", url, grpc_port);
        info!("Attempting connection to: {}", url);
        let channel = Channel::from_shared(url)?
            .tls_config(tls)?
            .connect()
            .await?;
        let client = pb::node_client::NodeClient::new(channel);

        Ok(Self { client })
    }

    pub async fn get_info(&mut self) -> Result<pb::GetinfoResponse> {
        let response = self.client.getinfo(pb::GetinfoRequest {}).await?;
        Ok(response.into_inner())
    }

    pub async fn get_route(&mut self, id: &str, amount_msat: u64) -> Result<pb::GetrouteResponse> {
        let destination = hex::decode(id)?;
        let response = self
            .client
            .get_route(pb::GetrouteRequest {
                amount_msat: Some(pb::Amount { msat: amount_msat }),
                cltv: None,
                exclude: vec![],
                fromid: None,
                fuzzpercent: None,
                id: destination,
                maxhops: None,
                riskfactor: 0u64,
            })
            .await?;
        Ok(response.into_inner())
    }

    pub async fn keysend_with_route_hint(
        &mut self,
        _id: &str,
        _amt: u64,
        _last_hop_id: &str,
        _scid: &str,
        _feebase: u64,
        _expirydelta: u32,
        _feeprop: u32,
    ) -> Result<pb::KeysendResponse> {
        let mut routehints = pb::RoutehintList { hints: vec![] };
        let mut hint0 = pb::Routehint { hops: vec![] };

        // remy
        //let id = "02970cbacbbd9871d532f08460612a0d6fd2e78ac2b95bd83f7048c7a14a9113e4";
        //let amt_msat = 5000;
        //let destination = hex::decode(id)?;

        // deran
        let id = "02e2b4a7836574e9504e86131ddb56f105feba1ebef2151684ccfbabf0114a1c77";
        let amt_msat = 123000;
        let destination = hex::decode(id)?;

        // game b 1 -> game b 3
        let gameb_1_id = "023d70f2f76d283c6c4e58109ee3a2816eb9d8feb40b23d62469060a2b2867b77f";
        let gameb_1_scid = "750452x1749x1";
        let gameb_1_feebase = 0;
        let gameb_1_delta = 100;
        let gameb_1_feeprop = 3000;
        let hop0 = pb::RouteHop {
            id: hex::decode(gameb_1_id)?,
            short_channel_id: gameb_1_scid.to_string(),
            feebase: Some(_amount(gameb_1_feebase)),
            expirydelta: gameb_1_delta,
            feeprop: gameb_1_feeprop,
        };

        // game b 3 -> deran
        let gameb_3_id = "02736e7dad83d7205826649fc17db672ce08f8e87a2b47c7785ccbf79f24e91db0";
        let gameb_3_scid = to_cln(1099539677185);
        let gameb_3_feebase = 0;
        let gameb_3_delta = 0;
        let gameb_3_feeprop = 0;
        let hop1 = pb::RouteHop {
            id: hex::decode(gameb_3_id)?,
            short_channel_id: gameb_3_scid,
            feebase: Some(_amount(gameb_3_feebase)),
            expirydelta: gameb_3_delta,
            feeprop: gameb_3_feeprop,
        };


        hint0.hops.push(hop0);
        hint0.hops.push(hop1);
        routehints.hints.push(hint0);
        let response = self
            .client
            .key_send(pb::KeysendRequest {
                destination,
                amount_msat: Some(_amount(amt_msat)),
                routehints: Some(routehints),
                ..Default::default()
            })
            .await?;
        Ok(response.into_inner())
    }
}

fn _amount_or_any(msat: u64) -> Option<pb::AmountOrAny> {
    Some(pb::AmountOrAny {
        value: Some(pb::amount_or_any::Value::Amount(_amount(msat))),
    })
}
fn _amount_or_all(msat: u64) -> Option<pb::AmountOrAll> {
    Some(pb::AmountOrAll {
        value: Some(pb::amount_or_all::Value::Amount(_amount(msat))),
    })
}
fn _amount(msat: u64) -> pb::Amount {
    pb::Amount { msat }
}
