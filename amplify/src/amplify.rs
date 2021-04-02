use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Campaigns {
    pub campaigns: Vec<Campaign>,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Campaign {
    pub group: Group,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Group {
    pub id: i32,
    pub donatable: bool,
    pub logo_url: String,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Organization {
    pub description: Option<String>,
    pub name: String,
    pub slug: String,
    pub address: Address,
    pub contact: Contact,
    pub causes: Vec<Cause>,
    // pub causes: Causes,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Address {
    pub address1: String,
    pub address2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub venue: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
pub struct Contact {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

// #[derive(Debug, Deserialize, Clone, PartialEq)]
// pub struct Causes {
//     pub causes: Vec<Cause>,
// }

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct Cause {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, PartialOrd)]
pub struct Amplify {
    pub name: String,
    pub description: Option<String>,
    pub categories: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zipcode: String,
    pub country: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub contact: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

impl From<Organization> for Amplify {
    fn from(item: Organization) -> Self {
        let categories: Vec<String> = item.causes.into_iter().map(|c| c.name).collect();
        Amplify {
            name: item.name,
            email: item.contact.email.unwrap_or_default(),
            address: format!(
                "{} {}",
                item.address.address1,
                item.address.address2.unwrap_or("".to_owned())
            ),
            city: item.address.city,
            state: item.address.state,
            zipcode: item.address.postal_code,
            country: item.address.country,
            phone: item.contact.phone,
            contact: item.contact.name,
            latitude: item.address.latitude,
            longitude: item.address.longitude,
            description: item.description,
            categories: categories.join(", "),
        }
    }
}
