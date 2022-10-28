// Non snake case for backwards compatibility
#![allow(non_snake_case)]

pub mod store {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use candid::{CandidType, Deserialize, Principal};
    use crate::types::assets::AssetHashes;
    use crate::types::storage::{Asset, Batch, Chunk};

    pub type Batches = HashMap<u128, Batch>;
    pub type Chunks = HashMap<u128, Chunk>;
    pub type Assets = HashMap<String, Asset>;

    #[derive(Default, Clone)]
    pub struct State {
        pub stable: StableState,
        pub runtime: RuntimeState,
    }

    #[derive(Default, CandidType, Deserialize, Clone)]
    pub struct StableState {
        pub user: Option<Principal>,
        pub assets: Assets,
    }

    #[derive(Default, Clone)]
    pub struct RuntimeState {
        pub chunks: Chunks,
        pub batches: Batches,
        pub asset_hashes: AssetHashes,
    }
}

pub mod assets {
    use std::clone::Clone;
    use ic_certified_map::{Hash, RbTree};

    #[derive(Default, Clone)]
    pub struct AssetHashes {
        pub tree: RbTree<String, Hash>,
    }
}

pub mod storage {
    use std::collections::HashMap;
    use candid::{Principal, CandidType, Int};
    use serde::Deserialize;
    use std::clone::Clone;
    use crate::types::http::HeaderField;
    use ic_certified_map::{Hash};

    #[derive(CandidType, Deserialize, Clone)]
    pub struct Chunk {
        pub batchId: u128,
        pub content: Vec<u8>,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct AssetEncoding {
        pub modified: Int,
        pub contentChunks: Vec<Vec<u8>>,
        pub totalLength: u128,
        pub sha256: Hash,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct AssetKey {
        // myimage.jpg
        pub name: String,
        // images
        pub folder: String,
        // /images/myimage.jpg
        pub fullPath: String,
        // ?token=1223-3345-5564-3333
        pub token: Option<String>,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct Asset {
        pub key: AssetKey,
        pub headers: Vec<HeaderField>,
        pub encoding: AssetEncoding,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct Batch {
        pub key: AssetKey,
        pub expiresAt: u64,
    }
}

pub mod interface {
    use candid::{CandidType, Deserialize};

    use crate::types::http::HeaderField;

    #[derive(CandidType)]
    pub struct InitUpload {
        pub batchId: u128,
    }

    #[derive(CandidType)]
    pub struct UploadChunk {
        pub chunkId: u128,
    }

    #[derive(CandidType, Deserialize)]
    pub struct CommitBatch {
        pub batchId: u128,
        pub headers: Vec<HeaderField>,
        pub chunkIds: Vec<u128>,
    }

    #[derive(CandidType, Deserialize)]
    pub struct Del {
        pub fullPath: String,
        pub token: Option<String>,
    }
}

pub mod http {
    use candid::{CandidType, Deserialize, Func};
    use serde_bytes::ByteBuf;

    #[derive(CandidType, Deserialize, Clone)]
    pub struct HeaderField(pub String, pub String);

    #[derive(CandidType, Deserialize, Clone)]
    pub struct HttpRequest {
        pub url: String,
        pub method: String,
        pub headers: Vec<HeaderField>,
        pub body: Vec<u8>,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct HttpResponse {
        pub body: Vec<u8>,
        pub headers: Vec<HeaderField>,
        pub status_code: u16,
        pub streaming_strategy: Option<StreamingStrategy>,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub enum StreamingStrategy {
        Callback {
            token: StreamingCallbackToken,
            callback: Func,
        },
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct StreamingCallbackToken {
        pub fullPath: String,
        pub token: Option<String>,
        pub headers: Vec<HeaderField>,
        pub sha256: Option<ByteBuf>,
        pub index: usize,
    }

    #[derive(CandidType, Deserialize, Clone)]
    pub struct StreamingCallbackHttpResponse {
        pub body: Vec<u8>,
        pub token: Option<StreamingCallbackToken>,
    }
}

// TODO: delete after migration from Motoko to Rust

pub mod migration {
    use candid::{CandidType, Deserialize, Principal};
    use crate::types::storage::Asset;

    #[derive(CandidType, Deserialize)]
    pub struct UpgradeState {
        pub user: Option<Principal>,
        pub entries: Option<Vec<(String, Asset)>>,
    }
}