import { Address } from "@graphprotocol/graph-ts";
import { ReserveToken, Transfer } from "../generated/OrderBook/ReserveToken";
import { ERC20 } from "../generated/schema";
import { toDisplay } from "./utils";

export function handleTransfer(event: Transfer): void {
  let token = ERC20.load(event.address.toHex());

  if (
    token &&
    (event.params.from == Address.zero() || event.params.to == Address.zero())
  ) {
    let reserveToken = ReserveToken.bind(Address.fromBytes(event.address));

    let totalSupply = reserveToken.try_totalSupply();

    if (!totalSupply.reverted) {
      let value = totalSupply.value;

      token.totalSupply = value;
      token.totalSupplyDisplay = toDisplay(value, event.address.toHex());

      token.save();
    }
  }
}
