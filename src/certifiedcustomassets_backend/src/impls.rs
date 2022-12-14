use candid::Int;
use ic_certified_map::{Hash};
use crate::types::assets::AssetHashes;
use crate::types::store::Assets;
use crate::types::storage::{Asset, AssetEncoding};
use sha2::{Digest, Sha256};
use ic_cdk::{api::{time}};

impl From<&Assets> for AssetHashes {
    fn from(assets: &Assets) -> Self {
        let mut asset_hashes = Self::default();

        for (_key, asset) in assets.iter() {
            asset_hashes.insert(asset);
        }

        asset_hashes
    }
}

impl AssetHashes {
    pub(crate) fn insert(&mut self, asset: &Asset) {
        self
            .tree
            .insert(asset.key.fullPath.clone(), asset.encoding.sha256);
    }

    pub(crate) fn delete(&mut self, full_path: &String) {
        self
            .tree
            .delete(full_path.as_bytes());
    }
}

impl From<&Vec<Vec<u8>>> for AssetEncoding {
    fn from(content_chunks: &Vec<Vec<u8>>) -> Self {
        let mut total_length: u128 = 0;
        let mut hasher = Sha256::new();

        // Calculate sha256 and total length
        for chunk in content_chunks.iter() {
            total_length += u128::try_from(chunk.len()).unwrap();

            hasher.update(chunk);
        }

        let sha256 = hasher.finalize().into();

        AssetEncoding {
            modified: Int::from(time()),
            contentChunks: content_chunks.clone(),
            totalLength: total_length,
            sha256
        }
    }
}