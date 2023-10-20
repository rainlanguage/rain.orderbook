use ethers::types::{Address, Bytes, U256};
use serde::{Deserialize, Serialize};
use serde_json::{Result, Value};

use crate::generated::NewExpressionFilter;
use crate::utils::hex_string_to_bytes;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExpressionJson {
    bytecode: Bytes,
    constants: Vec<U256>,
    min_outputs: Vec<U256>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderJson {
    pub owner: Address,
    pub handle_io: bool,
    pub evaluable: EvaluableJson,
    pub valid_inputs: Vec<IoJson>,
    pub valid_outputs: Vec<IoJson>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluableJson {
    interpreter: Address,
    store: Address,
    expression: Address,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IoJson {
    token: Address,
    decimals: u8,
    vault_id: U256,
}

impl NewExpressionJson {
    pub fn from_event(event_data: NewExpressionFilter) -> NewExpressionJson {
        NewExpressionJson {
            bytecode: event_data.bytecode.clone(),
            constants: event_data.constants.clone(),
            min_outputs: event_data.min_outputs.clone(),
        }
    }

    pub fn _from_json_string(json_data: &String) -> anyhow::Result<NewExpressionJson> {
        let parsed_json: Result<Value> = serde_json::from_str(json_data);

        match parsed_json {
            Ok(data) => {
                println!("parsed_expression: {:?}", data);
                let obj = data.as_object().unwrap();

                let bytecode =
                    hex_string_to_bytes(obj.get("bytecode").unwrap().as_str().unwrap()).unwrap();

                let constants =
                    _array_to_vec_256(obj.get("constants").unwrap().as_array().unwrap());

                let min_outputs =
                    _array_to_vec_256(obj.get("minOutputs").unwrap().as_array().unwrap());

                Ok(NewExpressionJson {
                    bytecode,
                    constants,
                    min_outputs,
                })
            }
            Err(err) => Err(anyhow::anyhow!("parse failed: {}", err)),
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize struct to JSON")
    }
}

fn _array_to_vec_256(values: &Vec<Value>) -> Vec<U256> {
    let resp: Vec<U256> = values
        .iter()
        .map(|data| U256::from_str_radix(data.as_str().unwrap(), 16).unwrap())
        .collect();

    return resp;
}
