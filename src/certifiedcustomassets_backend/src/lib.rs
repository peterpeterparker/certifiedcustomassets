mod types;
mod impls;
mod store;
mod env;
mod utils;
mod cert;
mod http;

use ic_cdk::api::management_canister::main::{CanisterIdRecord, deposit_cycles};
use ic_cdk_macros::{init, update, pre_upgrade, post_upgrade, query};
use ic_cdk::api::{canister_balance128, caller, trap};
use ic_cdk::export::candid::{candid_method, export_service};
use ic_cdk::{storage, id, api};
use candid::{decode_args, Principal};
use std::cell::RefCell;
use std::collections::HashMap;
use ic_certified_map::{labeled, labeled_hash, AsHashTree, Hash, RbTree};
use serde_bytes::ByteBuf;
use crate::cert::{update_certified_data};
use crate::http::{build_headers, create_token, streaming_strategy};

use crate::store::{commit_batch, create_batch, create_chunk, delete_asset, get_asset, get_asset_for_url, get_keys};
use crate::utils::{principal_not_equal, is_manager};
use crate::types::{interface::{InitUpload, UploadChunk, CommitBatch, Del}, store::{State, StableState, RuntimeState, Assets}, storage::{AssetKey, Chunk, Asset, AssetEncoding}, http::{HttpRequest, HttpResponse, HeaderField, StreamingStrategy, StreamingCallbackToken, StreamingCallbackHttpResponse}};
use crate::types::{migration::{UpgradeState}};
use crate::types::assets::AssetHashes;

thread_local! {
    static STATE: RefCell<State> = RefCell::default();
}

#[init]
fn init() {
    STATE.with(|state| {
        *state.borrow_mut() = State {
            stable: StableState {
                user: Some(Principal::anonymous()),
                assets: HashMap::new(),
            },
            runtime: RuntimeState {
                chunks: HashMap::new(),
                batches: HashMap::new(),
                asset_hashes: AssetHashes::default(),
            },
        };
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    STATE.with(|state| storage::stable_save((&state.borrow().stable, )).unwrap());
}

#[post_upgrade]
fn post_upgrade() {
    let (stable, ): (StableState, ) = ic_cdk::storage::stable_restore().unwrap();

    let asset_hashes = AssetHashes::from(&stable.assets);

    // Populate state
    STATE.with(|state| *state.borrow_mut() = State {
        stable,
        runtime: RuntimeState {
            chunks: HashMap::new(),
            batches: HashMap::new(),
            asset_hashes: asset_hashes.clone()
        },
    });

    update_certified_data(&asset_hashes);
}

fn upgrade_assets(UpgradeState {entries, user: _}: UpgradeState) -> Assets {
    match entries {
        None => HashMap::new(),
        Some(e) => e.into_iter().collect()
    }
}

//
// Http
//

#[query]
#[candid_method(query)]
fn http_request(HttpRequest { method, url, headers: _, body: _ }: HttpRequest) -> HttpResponse {
    if method != "GET" {
        return HttpResponse {
            body: "Method Not Allowed.".as_bytes().to_vec(),
            headers: Vec::new(),
            status_code: 405,
            streaming_strategy: None,
        };
    }

    let result = get_asset_for_url(url);

    match result {
        Ok(asset) => {
            let headers = build_headers(&asset);
            let Asset { key, headers: _, encoding } = asset;

            match headers {
                Ok(headers) => HttpResponse {
                    body: encoding.contentChunks[0].clone(),
                    headers: headers.clone(),
                    status_code: 200,
                    streaming_strategy: streaming_strategy(key, &encoding, &headers),
                },
                Err(err) => HttpResponse {
                    body: ["Permission denied. Invalid headers. ", err].join("").as_bytes().to_vec(),
                    headers: Vec::new(),
                    status_code: 405,
                    streaming_strategy: None,
                }
            }
        }
        Err(err) => HttpResponse {
            body: ["Permission denied. Could not perform this operation. ", err].join("").as_bytes().to_vec(),
            headers: Vec::new(),
            status_code: 405,
            streaming_strategy: None,
        }
    }
}

#[query]
#[candid_method(query)]
fn http_request_streaming_callback(StreamingCallbackToken { token, headers, index, sha256: _, fullPath }: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    let result = get_asset(fullPath, token);

    match result {
        Err(err) => trap(&*["Streamed asset not found: ", err].join("")),
        Ok(asset) => {
            return StreamingCallbackHttpResponse {
                token: create_token(asset.key, index, &asset.encoding, &headers),
                body: asset.encoding.contentChunks[index].clone(),
            };
        }
    }
}

//
// Upload
//

#[allow(non_snake_case)]
#[candid_method(update)]
#[update]
fn initUpload(key: AssetKey) -> InitUpload {
    let user: Principal = STATE.with(|state| state.borrow().stable.user).unwrap();

    if principal_not_equal(caller(), user) {
        trap("User does not have the permission to upload data.");
    }

    let batchId: u128 = create_batch(key);
    return InitUpload { batchId };
}

#[allow(non_snake_case)]
#[candid_method(update)]
#[update]
fn uploadChunk(chunk: Chunk) -> UploadChunk {
    let user: Principal = STATE.with(|state| state.borrow().stable.user).unwrap();

    if principal_not_equal(caller(), user) {
        trap("User does not have the permission to a upload any chunks of content.");
    }

    let result = create_chunk(chunk);

    match result {
        Ok(chunk_id) => { UploadChunk { chunkId: chunk_id } }
        Err(error) => trap(error)
    }
}

#[allow(non_snake_case)]
#[candid_method(update)]
#[update]
fn commitUpload(commit: CommitBatch) {
    let user: Principal = STATE.with(|state| state.borrow().stable.user).unwrap();

    if principal_not_equal(caller(), user) {
        trap("User does not have the permission to commit an upload.");
    }

    let result = commit_batch(commit);

    match result {
        Ok(_) => (),
        Err(error) => trap(error)
    }
}

//
// List and delete
//

#[allow(non_snake_case)]
#[candid_method(query)]
#[query]
fn list(folder: Option<String>) -> Vec<AssetKey> {
    let user: Principal = STATE.with(|state| state.borrow().stable.user).unwrap();

    if principal_not_equal(caller(), user) {
        trap("User does not have the permission to list the assets.");
    }

    get_keys(folder)
}

#[allow(non_snake_case)]
#[candid_method(update)]
#[update]
fn del(param: Del) {
    let user: Principal = STATE.with(|state| state.borrow().stable.user).unwrap();

    if principal_not_equal(caller(), user) {
        trap("User does not have the permission to delete an asset.");
    }

    let result = delete_asset(param);

    match result {
        Ok(_) => (),
        Err(error) => trap(&*["Asset cannot be deleted: ", error].join(""))
    }
}

//
// Canister mgmt
//

#[allow(non_snake_case)]
#[candid_method(update)]
#[update]
async fn transferFreezingThresholdCycles() {
    let caller = caller();

    if !is_manager(caller) {
        trap("Unauthorized access. Caller is not a manager.");
    }

    // TODO: determine effective threshold - how few cycles should be retained before deleting the canister?
    // use freezing_threshold_in_cycles? - https://github.com/dfinity/interface-spec/pull/18/files
    // actually above PR was ultimately deleted? - https://forum.dfinity.org/t/minimal-cycles-to-delete-canister/15926

    // Source: https://forum.dfinity.org/t/candid-nat-to-u128/16016
    let balance: u128 = canister_balance128();
    let cycles: u128 = balance - 100_000_000_000u128;

    if cycles > 0 {
        let arg_deposit = CanisterIdRecord { canister_id: caller };
        deposit_cycles(arg_deposit, cycles).await.unwrap();
    }
}

#[allow(non_snake_case)]
#[candid_method(query)]
#[query]
fn cyclesBalance() -> u128 {
    let caller = caller();
    let user: Principal = STATE.with(|state| state.borrow().stable.user).unwrap();

    if !is_manager(caller) && principal_not_equal(caller, user) {
        trap("No permission to read the balance of the cycles.");
    }

    canister_balance128()
}

// Generate did files

#[query(name = "__get_candid_interface_tmp_hack")]
fn export_candid() -> String {
    export_service!();
    __export_service()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_candid() {
        use std::env;
        use std::fs::write;
        use std::path::PathBuf;

        let dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let dir = dir.parent().unwrap().parent().unwrap().join("src").join("certifiedcustomassets_backend");
        write(dir.join("certifiedcustomassets_backend.did"), export_candid()).expect("Write failed.");
    }
}