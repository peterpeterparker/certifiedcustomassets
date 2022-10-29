use crate::STATE;
use crate::cert::{build_asset_certificate_header};
use crate::types::{http::{HeaderField}, storage::{Asset}, store::{RuntimeState}};

pub fn build_certified_headers(asset: &Asset) -> Result<Vec<HeaderField>, &'static str> {
    STATE.with(|state| build_certified_headers_impl(asset, &state.borrow().runtime))
}

fn build_certified_headers_impl(Asset { key, headers, encoding: _ }: &Asset, state: &RuntimeState) -> Result<Vec<HeaderField>, &'static str> {
    let mut certified_headers = headers.clone();

    let certificate_header = build_asset_certificate_header(&state.asset_hashes, key.fullPath.clone());

    match certificate_header {
        Err(err) => Err(err),
        Ok(certificate_header) => {
            certified_headers.push(certificate_header);
            Ok(certified_headers)
        }
    }
}