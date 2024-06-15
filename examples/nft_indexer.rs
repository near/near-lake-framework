//! This is a more complex real-life example of how to use the NEAR Lake Framework.
//!
//! It is going to follow the network and watch for the Events according to the
//! [Events Format][1]. It will monitor for nft_mint events from the known
//! marketplaces, such as Mintbase and Paras, and index them to print in the terminal.
//!
//! [1]: https://nomicon.io/Standards/EventsFormat
use near_lake_framework::near_lake_primitives;
use regex::Regex;

use once_cell::sync::Lazy;

static MINTBASE_STORE_REGEXP: Lazy<regex::Regex> =
    Lazy::new(|| Regex::new(r"^*.mintbase\d+.near$").unwrap());

fn main() -> anyhow::Result<()> {
    eprintln!("Starting...");
    // Lake Framework start boilerplate
    near_lake_framework::LakeBuilder::default()
        .testnet()
        .start_block_height(112205773)
        .build()?
        .run(handle_block)?; // developer-defined async function that handles each block
    Ok(())
}

async fn handle_block(block: near_lake_primitives::block::Block) -> anyhow::Result<()> {
    // Indexing lines START
    let nfts: Vec<NFTReceipt> = block
        .events() // fetching all the events that occurred in the block
        .values()
        .flatten()
        .filter(|event| event.standard() == "nep171")
        .filter(|event| event.event() == "nft_mint") // filter them by "nft_mint" event only
        .filter_map(|event| parse_event(event))
        .collect();
    // Indexing lines END

    if !nfts.is_empty() {
        println!("We caught freshly minted NFTs!\n{:#?}", nfts);
    }
    Ok(())
}

// ================================================================
// The following lines define structures and methods that support
// the goal of indexing NFT MINT events and printing links to newly
// created NFTs.
// These lines are not related to the NEAR Lake Framework.
// This logic is developer-defined and tailored to their indexing needs.
// ================================================================

/// Parses the given event to extract NFT data for known Marketplaces (Mintbase and Paras).
///
/// The function parses the event data to extract the owner and link to the NFT, then filters out any
/// Marketplaces or contracts that it doesn't know how to parse. The resulting NFT data is returned
/// as an `Option<NFTReceipt>`. Note that the logic used in this function is specific to the needs
/// of this application and does not relate to the Lake Framework.
///
/// # Arguments
///
/// * `event` - The event to parse for NFT data.
///
/// # Returns
///
/// An `Option<NFTReceipt>` containing the extracted NFT data, or `None` if the event data could not
/// be parsed.
fn parse_event(event: &near_lake_primitives::events::Event) -> Option<NFTReceipt> {
    let marketplace = {
        if MINTBASE_STORE_REGEXP.is_match(event.related_receipt_receiver_id().as_str()) {
            Marketplace::Mintbase
        } else if event.related_receipt_receiver_id().as_str() == "x.paras.near" {
            Marketplace::Paras
        } else {
            Marketplace::Unknown
        }
    };

    if let Some(event_data) = event.data() {
        if let Some(nfts) = marketplace
            .convert_event_data_to_nfts(event_data.clone(), event.related_receipt_receiver_id())
        {
            Some(NFTReceipt {
                receipt_id: event.related_receipt_id().to_string(),
                marketplace_name: marketplace.name(),
                nfts,
            })
        } else {
            None
        }
    } else {
        None
    }
}

enum Marketplace {
    Mintbase,
    Paras,
    Unknown,
}

impl Marketplace {
    fn name(&self) -> String {
        match self {
            Self::Mintbase => "Mintbase".to_string(),
            Self::Paras => "Paras".to_string(),
            Self::Unknown => "Unknown".to_string(),
        }
    }
    fn convert_event_data_to_nfts(
        &self,
        event_data: serde_json::Value,
        receiver_id: &near_lake_primitives::near_primitives::types::AccountId,
    ) -> Option<Vec<NFT>> {
        match self {
            Self::Mintbase => Some(self.mintbase(event_data, receiver_id)),
            Self::Paras => Some(self.paras(event_data, receiver_id)),
            Self::Unknown => None,
        }
    }

    fn paras(
        &self,
        event_data: serde_json::Value,
        receiver_id: &near_lake_primitives::near_primitives::types::AccountId,
    ) -> Vec<NFT> {
        let paras_event_data = serde_json::from_value::<Vec<NftMintLog>>(event_data)
            .expect("Failed to parse NftMintLog");

        paras_event_data
            .iter()
            .map(|nft_mint_log| NFT {
                owner: nft_mint_log.owner_id.clone(),
                links: nft_mint_log
                    .token_ids
                    .iter()
                    .map(|token_id| {
                        format!(
                            "https://paras.id/token/{}::{}/{}",
                            receiver_id.to_string(),
                            token_id.split(":").collect::<Vec<&str>>()[0],
                            token_id,
                        )
                    })
                    .collect(),
            })
            .collect()
    }

    fn mintbase(
        &self,
        event_data: serde_json::Value,
        receiver_id: &near_lake_primitives::near_primitives::types::AccountId,
    ) -> Vec<NFT> {
        let mintbase_event_data = serde_json::from_value::<Vec<NftMintLog>>(event_data)
            .expect("Failed to parse NftMintLog");

        mintbase_event_data
            .iter()
            .map(|nft_mint_log| NFT {
                owner: nft_mint_log.owner_id.clone(),
                links: vec![format!(
                    "https://mintbase.io/contract/{}/token/{}",
                    receiver_id.to_string(),
                    nft_mint_log.token_ids[0]
                )],
            })
            .collect()
    }
}

// We are allowing the dead_code lint because not all fields of the structures are used
// However, they are printed to the terminal for debugging purposes.
#[allow(dead_code)]
#[derive(Debug)]
struct NFTReceipt {
    receipt_id: String,
    marketplace_name: String,
    nfts: Vec<NFT>,
}

// We are allowing the dead_code lint because not all fields of the structures are used
// However, they are printed to the terminal for debugging purposes.
#[allow(dead_code)]
#[derive(Debug)]
struct NFT {
    owner: String,
    links: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct NftMintLog {
    owner_id: String,
    token_ids: Vec<String>,
    // There is also a `memo` field, but it is not used in this example
    // memo: Option<String>,
}
