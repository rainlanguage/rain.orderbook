import { Bytes } from "@graphprotocol/graph-ts";
import { getKeccak256FromBytes } from "@rainprotocol/subgraph-utils";

// Orderbook: TakeOrder
export let TAKE_ORDER_EVENT_TOPIC = getKeccak256FromBytes(
  Bytes.fromUTF8(
    "TakeOrder(address,((address,bool,(address,address,address),(address,uint8,uint256)[],(address,uint8,uint256)[]),uint256,uint256,(address,uint256[],bytes)[]),uint256,uint256)"
  )
).toHexString();

// Orderbook: Clear
export let CLEAR_EVENT_TOPIC = getKeccak256FromBytes(
  Bytes.fromUTF8(
    "Clear(address,(address,bool,(address,address,address),(address,uint8,uint256)[],(address,uint8,uint256)[]),(address,bool,(address,address,address),(address,uint8,uint256)[],(address,uint8,uint256)[]),(uint256,uint256,uint256,uint256,uint256,uint256))"
  )
).toHexString();

// Orderbook: AfterClear
export let AFTER_CLEAR_EVENT_TOPIC = getKeccak256FromBytes(
  Bytes.fromUTF8("AfterClear(address,(uint256,uint256,uint256,uint256))")
).toHexString();

// ExpressionDeployer: NewExpression
export let NEW_EXPRESSION_EVENT_TOPIC = getKeccak256FromBytes(
  Bytes.fromUTF8("NewExpression(address,bytes,uint256[],uint256[])")
).toHexString();
