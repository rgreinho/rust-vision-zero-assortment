use amplify::amplify::{Amplify, Campaign, Campaigns, Organization};
use eyre::Result;
use futures::stream::futures_unordered::FuturesUnordered;
use futures::stream::StreamExt;
use regex::Regex;
// use std::io;

// const SEARCH_URL, &str = "https://api.givegab.com/v1/campaigns";
static SEARCH_URL: &'static str = "https://api.givegab.com/v1/campaigns";
static DETAILS_URL: &'static str = "https://www.amplifyatx.org/organizations";

#[tokio::main]
pub async fn main() -> Result<()> {
    // Create a reqwest client.
    let client = reqwest::Client::new();

    // Prepare the requests to collect the campaigns.
    // We are supposed to collect 35 pages of campaigns, but for now it does not work.
    let get_campaign_tasks = (0..1)
        .into_iter()
        .map(|i| {
            client
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
                    ("page", &i.to_string()),
                ])
                .send()
        })
        .collect::<FuturesUnordered<_>>();

    // Collect the responses.
    let campaign_responses = get_campaign_tasks
        .map(|r| r.unwrap())
        .collect::<Vec<_>>()
        .await;

    // Deserialize them.
    let campaigns_json_tasks = campaign_responses
        .into_iter()
        .map(|r| r.json::<Campaigns>())
        .collect::<FuturesUnordered<_>>();

    // Collect Campaigns.
    let campaigns_collection = campaigns_json_tasks
        .map(|r| r.unwrap())
        .collect::<Vec<_>>()
        .await;
    println!("{} campaigns pages received", campaigns_collection.len());

    // Flatten them to make it easier to manipulate later on.
    let campaigns: Vec<Campaign> = campaigns_collection
        .into_iter()
        .map(|c| c.campaigns)
        .flatten()
        .collect();
    println!("{} campaigns found", campaigns.len());

    // Fetch detail pages.
    let get_details_tasks = campaigns
        .into_iter()
        .map(|c| {
            client
                .get(format!("{}/{}", DETAILS_URL, c.group.slug))
                .send()
        })
        .collect::<FuturesUnordered<_>>();

    // Collect the responses.
    // I need to put something like a buffer_unordered(8) somewhere here.
    let detail_responses = get_details_tasks
        .map(|r| r.unwrap())
        .collect::<Vec<_>>()
        .await;
    println!("{} campaigns collected", &detail_responses.len());

    // Extract the JSON representing the organization from the HTLM of the page
    // and deserialize it.
    let mut organizations: Vec<Organization> = Vec::new();
    let re = Regex::new(r#"var org = new app.Group\((.*)\);"#).unwrap();
    for response in detail_responses {
        // let url = response.clone().url();
        // let path = url.path().clone();
        let detail_page = response.text().await?;
        match re.captures(&detail_page) {
            Some(c) => {
                let org_json = c.get(1).unwrap().as_str();
                // dbg!(&org_json);
                let org: Organization = serde_json::from_str(&org_json)?;
                println!("Organization {} added", &org.name);
                organizations.push(org);
            }
            None => println!("No details for {}", "path"),
        };
    }

    println!();
    println!("==================================================================");
    println!("{} organizations to serialize", organizations.len());
    println!("==================================================================");
    println!();

    // Write the CSV file.
    let amplify: Vec<Amplify> = organizations.into_iter().map(Amplify::from).collect();
    // let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut wtr = csv::Writer::from_path("amplify-rust.csv")?;
    for record in amplify {
        wtr.serialize(record)?;
    }
    wtr.flush()?;

    // Return the value.
    Ok(())
}
