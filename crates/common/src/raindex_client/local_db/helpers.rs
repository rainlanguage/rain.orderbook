use super::LocalDbError;
use alloy::primitives::U256;
use rain_math_float::Float;

pub fn patch_deposit_amounts_with_decimals(
    decoded_events: serde_json::Value,
    decimals_by_addr: &std::collections::HashMap<String, u8>,
) -> Result<serde_json::Value, LocalDbError> {
    let events = decoded_events.as_array().ok_or_else(|| {
        LocalDbError::CustomError("Decoded events should be an array".to_string())
    })?;

    let mut patched = Vec::with_capacity(events.len());
    for ev in events.iter() {
        let mut ev_clone = ev.clone();
        let event_type = ev.get("event_type").and_then(|v| v.as_str()).unwrap_or("");
        if event_type == "DepositV2" {
            let obj = ev_clone.as_object_mut().ok_or_else(|| {
                LocalDbError::CustomError("Event should be an object".to_string())
            })?;
            let dd = obj
                .get_mut("decoded_data")
                .and_then(|v| v.as_object_mut())
                .ok_or_else(|| {
                    LocalDbError::CustomError("Missing decoded_data in DepositV2".to_string())
                })?;

            let token = dd.get("token").and_then(|v| v.as_str()).ok_or_else(|| {
                LocalDbError::CustomError("Missing token in DepositV2".to_string())
            })?;
            let token_key = token.to_ascii_lowercase();
            let decimals = decimals_by_addr.get(&token_key).ok_or_else(|| {
                LocalDbError::CustomError(format!(
                    "Missing decimals for token {} required to compute deposit_amount",
                    token
                ))
            })?;

            let amt_hex = dd
                .get("deposit_amount_uint256")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    LocalDbError::CustomError(
                        "Missing deposit_amount_uint256 in DepositV2".to_string(),
                    )
                })?;
            let digits = amt_hex.strip_prefix("0x").unwrap_or(amt_hex);
            let amount = U256::from_str_radix(digits, 16).map_err(|e| {
                LocalDbError::CustomError(format!(
                    "Invalid deposit_amount_uint256 '{}': {}",
                    amt_hex, e
                ))
            })?;

            let amount_float = Float::from_fixed_decimal(amount, *decimals).map_err(|e| {
                LocalDbError::CustomError(format!(
                    "Float conversion failed for deposit_amount (token {}, decimals {}): {}",
                    token, decimals, e
                ))
            })?;

            dd.insert(
                "deposit_amount".to_string(),
                serde_json::Value::String(amount_float.as_hex()),
            );
        }

        patched.push(ev_clone);
    }

    Ok(serde_json::Value::Array(patched))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn patch_success() {
        let decoded = serde_json::json!([
            {
                "event_type": "DepositV2",
                "decoded_data": {
                    "sender": "0x0000000000000000000000000000000000000001",
                    "token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "vault_id": "0x1",
                    "deposit_amount_uint256": "0xfa0"
                }
            }
        ]);

        let mut map = std::collections::HashMap::new();
        map.insert(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
            6u8,
        );

        let patched = patch_deposit_amounts_with_decimals(decoded, &map).unwrap();
        let arr = patched.as_array().unwrap();
        let dd = &arr[0]["decoded_data"];

        let expected = Float::from_fixed_decimal(U256::from(4000u64), 6)
            .unwrap()
            .as_hex();
        assert_eq!(dd["deposit_amount"], expected);
    }

    #[test]
    fn patch_missing_decimals_errors() {
        let decoded = serde_json::json!([
            {
                "event_type": "DepositV2",
                "decoded_data": {
                    "sender": "0x0000000000000000000000000000000000000001",
                    "token": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                    "vault_id": "0x1",
                    "deposit_amount_uint256": "0xfa0"
                }
            }
        ]);
        let map: std::collections::HashMap<String, u8> = std::collections::HashMap::new();
        let res = patch_deposit_amounts_with_decimals(decoded, &map);
        assert!(res.is_err());
    }
}
