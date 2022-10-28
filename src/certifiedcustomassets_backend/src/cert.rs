use ic_certified_map::{labeled, labeled_hash, AsHashTree, Hash, RbTree};
use ic_cdk::api::{trap, data_certificate, set_certified_data};
use serde::Serialize;
use serde_cbor::ser::{Serializer};
use base64::{encode};

use crate::types::{assets::AssetHashes, http::HeaderField};

const LABEL_ASSETS: &[u8] = b"http_assets";

pub fn update_certified_data(asset_hashes: &AssetHashes) {
    let prefixed_root_hash = &labeled_hash(LABEL_ASSETS, &asset_hashes.tree.root_hash());
    set_certified_data(&prefixed_root_hash[..]);
}

pub fn make_asset_certificate_header(asset_hashes: &AssetHashes, asset_name: String) -> HeaderField {
    // TODO: Return result to get rid of trap
    let certificate = data_certificate().unwrap_or_else(|| {
        trap("data certificate is only available in query calls");
        unreachable!()
    });

    let witness = asset_hashes.tree.witness(asset_name.as_bytes());
    let tree = labeled(LABEL_ASSETS, witness);

    let mut serializer = Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    // TODO: Return result to get rid of trap
    tree.serialize(&mut serializer)
        .unwrap_or_else(|e| trap(&format!("failed to serialize a hash tree: {}", e)));

    HeaderField (
        "IC-Certificate".to_string(),
        format!(
            "certificate=:{}:, tree=:{}:",
            encode(&certificate),
            encode(&serializer.into_inner())
        ),
    )
}
