use ic_certified_map::{Hash};
use crate::types::assets::AssetHashes;
use crate::types::store::Assets;
use crate::types::storage::AssetEncoding;
use sha2::{Digest, Sha256};

impl From<&Assets> for AssetHashes {
    fn from(assets: &Assets) -> Self {
        let mut asset_hashes = Self::default();

        fn hash_bytes(AssetEncoding {modified: _, contentChunks, totalLength: _}: &AssetEncoding) -> Hash {
            let mut hasher = Sha256::new();
            for chunks in contentChunks.iter() {
                hasher.update(chunks);
            }
            hasher.finalize().into()
        }

        for (_key, asset) in assets.iter() {
            let bytes = hash_bytes(&asset.encoding);

            asset_hashes
                .0
                .insert(asset.key.fullPath.as_bytes().to_vec(), bytes);
        }

        asset_hashes
    }
}