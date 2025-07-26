use rain_orderbook_common::raindex_client::filters::vaults_filter::GetVaultsFilters;

pub fn filters_equal(a: &GetVaultsFilters, b: &GetVaultsFilters) -> bool {
    serde_json::to_string(a).unwrap() == serde_json::to_string(b).unwrap()
}
