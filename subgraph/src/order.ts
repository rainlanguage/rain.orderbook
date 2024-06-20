import { Bytes, ethereum } from "@graphprotocol/graph-ts";
import {
  AddOrderV2,
  AddOrderV2OrderStruct,
} from "../generated/OrderBook/OrderBook";
import { AddOrder, Order } from "../generated/schema";
import { vaultEntityId } from "./vault";

export function handleAddOrder(event: AddOrderV2): void {
  let order = new Order(event.params.orderHash);
  order.active = true;
  order.orderHash = event.params.orderHash;
  order.owner = event.params.sender;
  order.inputs = event.params.order.validInputs.map<Bytes>((input) =>
    vaultEntityId(input.vaultId, input.token)
  );
  order.outputs = event.params.order.validOutputs.map<Bytes>((output) =>
    vaultEntityId(output.vaultId, output.token)
  );
  order.nonce = event.params.order.nonce;
  order.orderStructEncoded =
    ethereum.encode(event.parameters[2].value) || Bytes.empty();
}
