mod providers;
use std::net::{IpAddr, Ipv4Addr};

use anyhow::Result;
use cloudflare::endpoints::dns::dns::{CreateDnsRecord, CreateDnsRecordParams, DnsContent, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord, UpdateDnsRecordParams};
use cloudflare::endpoints::zones::zone::{ListZones, ListZonesParams, Status};
use cloudflare::framework::Environment::Production;
use cloudflare::framework::auth::Credentials::UserAuthToken;
use cloudflare::framework::client::{ClientConfig, async_api::Client as AsyncClient};
type DnsRecord = (String, String, Ipv4Addr); // (record_id, record_name, record_content)
#[tokio::main]
async fn main() -> Result<()> {
    let cf_token = "ad74vvoDGK0M3aE5Lzgzy8aZDZsXvaYvHP5p0Hfn";
    // let cf_zone_id: Option<&str> = None;
    // get_public_ip().await;
    println!("Cloudflare Token: {}", cf_token);

    // let client = AsyncClient::new(
    //     UserAuthToken { token: cf_token.into() },
    //     ClientConfig::default(),
    //     Production,
    // )
    // .unwrap();
    // let zones_id = cf_get_active_zone_id(&client, None).await?;
    // println!("Active Zone IDs: {:?}", zones_id); //TODO: 可能從cli、tui、gui中選取一個
    // let ac_ips = cf_get_dns_records(&client, &zones_id[0], Some("ddns1.duacodie.com")).await?;
    // println!("Active DNS Records (A): {:?}", ac_ips);
    // let tmp = cf_update_dns_record(&client, &zones_id[0], "ddns.duacodie.com", &ac_ips[0].0, get_public_ip().await?, None).await?;
    // println!("Updated DNS Record: {:#?}", tmp);
    // cf_create_dns_record(&client, &zones_id[0], "ddns1.duacodie.com", get_public_ip().await?, None).await?;
    // let ac_ips = cf_get_dns_records(&client, &zones_id[0], Some("ddns1.duacodie.com")).await?;
    // println!("Active DNS Records (A): {:?}", ac_ips);
    // cf_delete_dns_record(&client, &zones_id[0], &ac_ips[0].0).await?;

    Ok(())
}

#[allow(unused)]
async fn get_public_ip() -> Result<Ipv4Addr> {
    match public_ip_address::perform_lookup(None).await {
        Ok(resp) => {
            if let IpAddr::V4(ip) = resp.ip {
                println!("Public IPv4 Address: {}", ip);
                Ok(ip)
            } else {
                Err(anyhow::anyhow!("Received an unexpected IP type"))
            }
        },
        Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
    }
}

// async fn cf_get_active_zone_id(
//     client: &AsyncClient,
//     zone_name: Option<&str>,
// ) -> Result<Vec<String>> { // 可能有多個active zone，回傳Vec<zone_id>
//     let params = ListZonesParams {
//         status: Some(Status::Active),
//         name: zone_name.map(|s| s.to_string()),
//         ..Default::default()
//     };

//     match client.request(&ListZones { params }).await {
//         Ok(response) => {
//             let zones: Vec<String> = response.result.into_iter().map(|zone| zone.id).collect();

//             Ok(zones)
//         }
//         Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
//     }
// }

// async fn cf_get_dns_records(
//     client: &AsyncClient,
//     zone_id: &str,
//     dns_name: Option<&str>,
// ) -> Result<Vec<DnsRecord>> {
//     let params =
//         ListDnsRecordsParams { name: dns_name.map(|s| s.to_string()), ..Default::default() };
//     match client.request(&ListDnsRecords { zone_identifier: zone_id, params }).await {
//         Ok(response) => {
//             let records: Vec<DnsRecord> = response
//                 .result
//                 .into_iter()
//                 .filter_map(|record| match record.content {
//                     DnsContent::A { content } => Some((record.id, record.name, content)),
//                     _ => None,
//                 })
//                 .collect();
//             Ok(records)
//         }
//         Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
//     }
// }

// async fn cf_update_dns_record(
//     client: &AsyncClient,
//     zone_id: &str,
//     dns_name: &str,
//     record_id: &str,
//     new_ip: Ipv4Addr,
//     proxied: Option<bool>,
// ) -> Result<DnsRecord> {
//     let params = UpdateDnsRecordParams {
//         content: DnsContent::A { content: new_ip },
//         proxied,
//         ttl: None,
//         name: dns_name,
//     };
//     let resp = client
//         .request(&UpdateDnsRecord {
//             zone_identifier: zone_id,
//             identifier: record_id,
//             params,
//         })
//         .await?;

//     let ip = match resp.result.content {
//         DnsContent::A { content } => content,
//         _ => return Err(anyhow::anyhow!("Record is not an A record")),
//     };

//     Ok((resp.result.id, resp.result.name, ip))
// }

// async fn cf_create_dns_record(
//     client: &AsyncClient,
//     zone_id: &str,
//     dns_name: &str,
//     new_ip: Ipv4Addr,
//     proxied: Option<bool>,
// ) -> Result<DnsRecord> {
//     let params = CreateDnsRecordParams {
//         ttl: None,
//         priority: None,
//         proxied,
//         name: dns_name,
//         content: DnsContent::A { content: new_ip },
//     };
//     let resp = client
//         .request(&CreateDnsRecord { zone_identifier: zone_id, params })
//         .await?;
//     let ip = match resp.result.content {
//         DnsContent::A { content } => content,
//         _ => return Err(anyhow::anyhow!("Record is not an A record")),
//     };
//     Ok((resp.result.id, resp.result.name, ip))
// }

// async fn cf_delete_dns_record(
//     client: &AsyncClient,
//     zone_id: &str,
//     record_id: &str,
// ) -> Result<bool> {
//     match client.request(&cloudflare::endpoints::dns::dns::DeleteDnsRecord{zone_identifier: zone_id,identifier: record_id}).await {
//         Ok(_) => {
//             Ok(true)
//         }
//         Err(e) => Err(anyhow::anyhow!("Error occurred: {}", e)),
//     }
// }