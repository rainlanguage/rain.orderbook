use ethers::types::{Address, Bytes, U256};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Result, Value};

use crate::generated::{Evaluable, Io, NewExpressionFilter, Order};
use crate::utils::hex_string_to_bytes;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewExpressionJson {
    bytecode: Bytes,
    constants: Vec<U256>,
    min_outputs: Vec<U256>,
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderJson {
    pub owner: Address,
    pub handle_io: bool,
    pub evaluable: EvaluableJson,
    pub valid_inputs: Vec<IoJson>,
    pub valid_outputs: Vec<IoJson>,
}

impl OrderJson {
    pub fn from_order(data: Order) -> OrderJson {
        let evaluable = EvaluableJson::from(data.evaluable);
        let valid_inputs = data
            .valid_inputs
            .iter()
            .map(|data| IoJson::from(data.clone()))
            .collect();

        let valid_outputs = data
            .valid_outputs
            .iter()
            .map(|data| IoJson::from(data.clone()))
            .collect();

        OrderJson {
            owner: data.owner,
            handle_io: data.handle_io,
            evaluable,
            valid_inputs,
            valid_outputs,
        }
    }

    pub fn _from_json_string(json_data: &String) -> anyhow::Result<OrderJson> {
        let parsed_json: Result<Value> = serde_json::from_str(json_data);

        match parsed_json {
            Ok(data) => {
                let obj = data.as_object().unwrap();

                let owner = Address::from_slice(
                    &hex_string_to_bytes(obj.get("owner").unwrap().as_str().unwrap()).unwrap(),
                );

                let handle_io = obj.get("handleIo").unwrap().as_bool().unwrap();

                let evaluable = EvaluableJson::_from_value(obj.get("evaluable").unwrap());

                let valid_inputs: Vec<IoJson> = obj
                    .get("validInputs")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|data| IoJson::_from_value(data))
                    .collect();

                let valid_outputs: Vec<IoJson> = obj
                    .get("validOutputs")
                    .unwrap()
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|data| IoJson::_from_value(data))
                    .collect();

                Ok(OrderJson {
                    owner,
                    handle_io,
                    evaluable,
                    valid_inputs,
                    valid_outputs,
                })
            }
            Err(err) => Err(anyhow::anyhow!("parse failed: {}", err)),
        }
    }

    pub fn to_json_string(&self) -> String {
        serde_json::to_string(&self).expect("Failed to serialize struct to JSON")
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluableJson {
    interpreter: Address,
    store: Address,
    expression: Address,
}

impl EvaluableJson {
    fn from(data: Evaluable) -> EvaluableJson {
        EvaluableJson {
            interpreter: data.interpreter,
            store: data.store,
            expression: data.expression,
        }
    }

    fn _from_value(value: &Value) -> EvaluableJson {
        let obj = value.as_object().unwrap();

        let interpreter = Address::from_slice(
            &hex_string_to_bytes(obj.get("interpreter").unwrap().as_str().unwrap()).unwrap(),
        );

        let store = Address::from_slice(
            &hex_string_to_bytes(obj.get("store").unwrap().as_str().unwrap()).unwrap(),
        );

        let expression = Address::from_slice(
            &hex_string_to_bytes(obj.get("expression").unwrap().as_str().unwrap()).unwrap(),
        );

        EvaluableJson {
            interpreter,
            store,
            expression,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IoJson {
    token: Address,
    decimals: QuotedU8,
    vault_id: U256,
}

impl IoJson {
    fn from(data: Io) -> IoJson {
        IoJson {
            token: data.token,
            decimals: QuotedU8(data.decimals),
            vault_id: data.vault_id,
        }
    }

    fn _from_value(value: &Value) -> IoJson {
        let obj = value.as_object().unwrap();

        let token = Address::from_slice(
            &hex_string_to_bytes(obj.get("token").unwrap().as_str().unwrap()).unwrap(),
        );

        let decimals: u8 = obj.get("decimals").unwrap().as_u64().unwrap() as u8;

        let vault_id =
            U256::from_str_radix(obj.get("vaultId").unwrap().as_str().unwrap(), 16).unwrap();

        IoJson {
            token,
            decimals: QuotedU8(decimals),
            vault_id,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct QuotedU8(u8);

impl Serialize for QuotedU8 {
    fn serialize<S>(&self, serializer: S) -> anyhow::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Format the u8 as a string enclosed in double quotes
        let quoted_value = format!("{}", self.0);

        // Serialize the quoted value as a string
        serializer.serialize_str(&quoted_value)
    }
}

fn _array_to_vec_256(values: &Vec<Value>) -> Vec<U256> {
    let resp: Vec<U256> = values
        .iter()
        .map(|data| U256::from_str_radix(data.as_str().unwrap(), 16).unwrap())
        .collect();

    return resp;
}
