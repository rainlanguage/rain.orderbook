use alloy::primitives::Address;
use serde_json::Value;
use std::collections::BTreeSet;
use std::str::FromStr;

pub fn collect_token_addresses(decoded_events: &Value) -> BTreeSet<Address> {
    let mut out = BTreeSet::new();

    let events = match decoded_events.as_array() {
        Some(arr) => arr,
        None => return out,
    };

    for event in events {
        let Some(event_type) = event.get("event_type").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(decoded) = event.get("decoded_data") else {
            continue;
        };

        match event_type {
            "DepositV2" | "WithdrawV2" => {
                if let Some(token_str) = decoded.get("token").and_then(|v| v.as_str()) {
                    if let Ok(addr) = Address::from_str(token_str) {
                        out.insert(addr);
                    }
                }
            }
            "AddOrderV3" | "RemoveOrderV3" => {
                if let Some(order) = decoded.get("order") {
                    if let Some(inputs) = order.get("valid_inputs").and_then(|v| v.as_array()) {
                        for io in inputs {
                            if let Some(token_str) = io.get("token").and_then(|v| v.as_str()) {
                                if let Ok(addr) = Address::from_str(token_str) {
                                    out.insert(addr);
                                }
                            }
                        }
                    }
                    if let Some(outputs) = order.get("valid_outputs").and_then(|v| v.as_array()) {
                        for io in outputs {
                            if let Some(token_str) = io.get("token").and_then(|v| v.as_str()) {
                                if let Ok(addr) = Address::from_str(token_str) {
                                    out.insert(addr);
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    out
}

pub fn collect_store_addresses(decoded_events: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();

    let events = match decoded_events.as_array() {
        Some(arr) => arr,
        None => return out,
    };

    for event in events {
        let Some(event_type) = event.get("event_type").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(decoded) = event.get("decoded_data") else {
            continue;
        };

        match event_type {
            "AddOrderV3" | "RemoveOrderV3" => {
                if let Some(store) = decoded
                    .get("order")
                    .and_then(|order| order.get("evaluable"))
                    .and_then(|eval| eval.get("store"))
                    .and_then(|v| v.as_str())
                {
                    out.insert(store.to_ascii_lowercase());
                }
            }
            "TakeOrderV3" => {
                if let Some(store) = decoded
                    .get("config")
                    .and_then(|cfg| cfg.get("order"))
                    .and_then(|order| order.get("evaluable"))
                    .and_then(|eval| eval.get("store"))
                    .and_then(|v| v.as_str())
                {
                    out.insert(store.to_ascii_lowercase());
                }
            }
            _ => {}
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn collects_from_deposit_and_withdraw() {
        let data = json!([
            {
                "event_type": "DepositV2",
                "decoded_data": {"token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}
            },
            {
                "event_type": "WithdrawV2",
                "decoded_data": {"token": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}
            }
        ]);

        let addrs = collect_token_addresses(&data);
        assert!(addrs
            .contains(&Address::from_str("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap()));
        assert!(addrs
            .contains(&Address::from_str("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap()));
        assert_eq!(addrs.len(), 2);
    }

    #[test]
    fn collects_from_add_and_remove_orders_inputs_outputs() {
        let data = json!([
            {
                "event_type": "AddOrderV3",
                "decoded_data": {
                    "order": {
                        "valid_inputs": [
                            {"token": "0x1111111111111111111111111111111111111111"}
                        ],
                        "valid_outputs": [
                            {"token": "0x2222222222222222222222222222222222222222"}
                        ]
                    }
                }
            },
            {
                "event_type": "RemoveOrderV3",
                "decoded_data": {
                    "order": {
                        "valid_inputs": [
                            {"token": "0x3333333333333333333333333333333333333333"}
                        ],
                        "valid_outputs": [
                            {"token": "0x1111111111111111111111111111111111111111"}
                        ]
                    }
                }
            }
        ]);

        let addrs = collect_token_addresses(&data);
        assert!(addrs
            .contains(&Address::from_str("0x1111111111111111111111111111111111111111").unwrap()));
        assert!(addrs
            .contains(&Address::from_str("0x2222222222222222222222222222222222222222").unwrap()));
        assert!(addrs
            .contains(&Address::from_str("0x3333333333333333333333333333333333333333").unwrap()));
        assert_eq!(addrs.len(), 3);
    }

    #[test]
    fn skips_missing_and_invalid() {
        let data = json!([
            {"event_type": "DepositV2", "decoded_data": {"token": null}},
            {"event_type": "WithdrawV2", "decoded_data": {}},
            {"event_type": "AddOrderV3", "decoded_data": {"order": {"valid_inputs": [{"token": 123}], "valid_outputs": []}}},
            {"event_type": "Unknown", "decoded_data": {"token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}
        ]);
        let addrs = collect_token_addresses(&data);
        assert!(addrs.is_empty());
    }

    #[test]
    fn collects_store_addresses() {
        let data = json!([
            {
                "event_type": "AddOrderV3",
                "decoded_data": {
                    "order": {
                        "evaluable": {"store": "0x1111111111111111111111111111111111111111"}
                    }
                }
            },
            {
                "event_type": "RemoveOrderV3",
                "decoded_data": {
                    "order": {
                        "evaluable": {"store": "0x2222222222222222222222222222222222222222"}
                    }
                }
            },
            {
                "event_type": "TakeOrderV3",
                "decoded_data": {
                    "config": {
                        "order": {
                            "evaluable": {"store": "0x3333333333333333333333333333333333333333"}
                        }
                    }
                }
            }
        ]);

        let stores = collect_store_addresses(&data);
        assert_eq!(stores.len(), 3);
        assert!(stores.contains("0x1111111111111111111111111111111111111111"));
        assert!(stores.contains("0x2222222222222222222222222222222222222222"));
        assert!(stores.contains("0x3333333333333333333333333333333333333333"));
    }
}
