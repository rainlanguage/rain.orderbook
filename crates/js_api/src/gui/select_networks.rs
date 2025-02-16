use std::collections::HashMap;

use super::*;
use rain_orderbook_app_settings::{gui::SelectNetwork, network::Network};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Tsify)]
pub struct SelectNetworks(HashMap<String, SelectNetwork>);
impl_all_wasm_traits!(SelectNetworks);

#[wasm_bindgen]
impl DotrainOrderGui {
    #[wasm_bindgen(js_name = "getSelectNetworks")]
    pub fn get_select_networks(&self) -> Result<SelectNetworks, GuiError> {
        let mut select_networks = HashMap::new();
        for (key, value) in Gui::parse_select_networks(self.dotrain_order.dotrain_yaml().documents)?
            .unwrap_or_default()
            .into_iter()
        {
            select_networks.insert(key, value);
        }
        Ok(SelectNetworks(select_networks))
    }

    #[wasm_bindgen(js_name = "isSelectNetworkSet")]
    pub fn is_select_network_set(&self) -> bool {
        self.selected_network.is_some()
    }

    #[wasm_bindgen(js_name = "getSelectedNetwork")]
    pub fn get_selected_network(&self) -> Result<String, GuiError> {
        self.selected_network
            .clone()
            .ok_or(GuiError::SelectNetworkNotSelected)
    }

    #[wasm_bindgen(js_name = "checkSelectNetwork")]
    pub fn check_select_network(&self) -> Result<(), GuiError> {
        let select_networks =
            Gui::parse_select_networks(self.dotrain_order.dotrain_yaml().documents)?;

        if select_networks.is_some() && self.selected_network.is_none() {
            return Err(GuiError::SelectNetworkNotSelected);
        }

        Ok(())
    }

    #[wasm_bindgen(js_name = "saveSelectNetwork")]
    pub fn save_select_network(&mut self, key: String) -> Result<(), GuiError> {
        let select_networks =
            Gui::parse_select_networks(self.dotrain_order.dotrain_yaml().documents)?
                .ok_or(GuiError::SelectNetworksNotSet)?;

        if !select_networks.contains_key(&key) {
            return Err(GuiError::SelectNetworkNotFound(key));
        }

        let network_keys =
            Network::parse_network_keys(self.dotrain_order.dotrain_yaml().documents.clone())?;
        if !network_keys.contains(&key) {
            return Err(GuiError::NetworkNotFound(key));
        }

        self.selected_network = Some(key);
        Ok(())
    }

    #[wasm_bindgen(js_name = "resetSelectNetwork")]
    pub fn reset_select_network(&mut self) {
        self.selected_network = None;
    }
}
