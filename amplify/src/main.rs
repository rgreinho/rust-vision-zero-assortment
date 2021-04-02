use amplify::amplify::{Amplify, Campaign, Campaigns, Organization};
use eyre::{eyre, Result};
use futures::stream::{self, StreamExt};
use regex::Regex;
// use std::io;

static SEARCH_URL: &'static str = "https://api.givegab.com/v1/campaigns";
static DETAILS_URL: &'static str = "https://www.amplifyatx.org/organizations";
const CONCURRENT_REQUESTS: usize = 50;

#[tokio::main]
pub async fn main() -> Result<()> {
    // Create a reqwest client.
    let client = reqwest::Client::new();

    // Prepare the requests to collect the campaigns.
    let get_campaign_tasks = stream::iter(0..=35)
        .map(|i| fetch_campaigns(&client, i))
        .buffer_unordered(CONCURRENT_REQUESTS);

    // Collect the responses.
    let campaign_collection = get_campaign_tasks
        .map(|r| r.unwrap_or_default())
        .collect::<Vec<_>>()
        .await;

    // Flatten them to make it easier to manipulate later on.
    let campaigns: Vec<Campaign> = campaign_collection.into_iter().flatten().collect();
    println!("{} campaigns found", campaigns.len());

    let get_details_tasks = stream::iter(campaigns)
        .map(|c| fetch_organizations(&client, c.group.slug.clone()))
        .buffer_unordered(CONCURRENT_REQUESTS);

    // Collect the responses.
    let organizations = get_details_tasks
        .map(|r| r.unwrap())
        .collect::<Vec<_>>()
        .await;
    println!("{} campaigns collected", &organizations.len());

    println!();
    println!("==================================================================");
    println!("{} organizations to serialize", organizations.len());
    println!("==================================================================");
    println!();

    // Write the CSV file.
    let mut amplify: Vec<Amplify> = organizations.into_iter().map(Amplify::from).collect();
    amplify.sort_unstable_by_key(|a| a.name.clone());
    // let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut wtr = csv::Writer::from_path("amplify-rust.csv")?;
    for record in amplify {
        wtr.serialize(record)?;
    }
    wtr.flush()?;

    // Return the value.
    Ok(())
}

async fn fetch_campaigns(client: &reqwest::Client, page: u8) -> Result<Vec<Campaign>> {
    let response = client
        .get(SEARCH_URL)
        .query(&[
            ("donatable", "true"),
            ("dog_campaign", ""),
            ("dog_id", "510"),
            ("use_new_search", "true"),
            ("visible_only", "true"),
            ("with", "address,dog_url,has_profile,stats,story_image_url"),
            ("sort_column", "alpha"),
            ("sort_order", "asc"),
            ("page", &page.to_string()),
        ])
        .send()
        .await?;

    let campaigns = response.json::<Campaigns>().await?;
    Ok(campaigns.campaigns)
}

async fn fetch_organizations(client: &reqwest::Client, slug: String) -> Result<Organization> {
    let re = Regex::new(r#"var org = new app.Group\((.*)\);"#).unwrap();
    let response = client
        .get(format!("{}/{}", DETAILS_URL, &slug))
        .send()
        .await?;
    let detail_page = response.text().await?;
    match re.captures(&detail_page) {
        Some(c) => {
            let org_json = c.get(1).unwrap().as_str();
            let org: Organization = match serde_json::from_str(&org_json) {
                Ok(o) => o,
                Err(e) => {
                    return Err(eyre!(
                        "cannot deserialize organization: {} -> {}",
                        e,
                        detail_page
                    ))
                }
            };

            return Ok(org);
        }
        None => {
            return Ok(Organization {
                ..Default::default()
            })
        } // None => return Err(eyre!("No details found")),
    };
}
