use ic_certified_map::{labeled, labeled_hash, AsHashTree, Hash, RbTree};
use serde::Serialize;
use crate::types::assets::AssetHashes;
use ic_cdk::api::{canister_balance128, caller, trap};
use crate::types::http::HeaderField;
use crate::types::storage::AssetKey;

const LABEL_ASSETS: &[u8] = b"http_assets";

pub fn update_root_hash(a: &AssetHashes) {
    let prefixed_root_hash = &labeled_hash(LABEL_ASSETS, &a.0.root_hash());
    ic_cdk::api::set_certified_data(&prefixed_root_hash[..]);
}

pub fn make_asset_certificate_header(asset_hashes: &AssetHashes, asset_name: String) -> HeaderField {
    let certificate = ic_cdk::api::data_certificate().unwrap_or_else(|| {
        trap("data certificate is only available in query calls");
        unreachable!()
    });
    let witness = asset_hashes.0.witness(asset_name.as_bytes());
    let tree = labeled(LABEL_ASSETS, witness);
    let mut serializer = serde_cbor::ser::Serializer::new(vec![]);
    serializer.self_describe().unwrap();
    tree.serialize(&mut serializer)
        .unwrap_or_else(|e| trap(&format!("failed to serialize a hash tree: {}", e)));
    HeaderField (
        "IC-Certificate".to_string(),
        format!(
            "certificate=:{}:, tree=:{}:",
            base64::encode(&certificate),
            base64::encode(&serializer.into_inner())
        ),
    )
}
