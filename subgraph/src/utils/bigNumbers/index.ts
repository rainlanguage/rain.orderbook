import { Address, BigDecimal, BigInt } from "@graphprotocol/graph-ts";
import { createToken } from "../entity/create";
import { getZeros } from "@rainprotocol/subgraph-utils";

export function toDisplay(amount: BigInt, token: string): BigDecimal {
  let erc20 = createToken(Address.fromString(token));
  if (erc20) {
    let denominator = BigInt.fromString(getZeros(erc20.decimals));
    return amount.toBigDecimal().div(denominator.toBigDecimal());
  }
  return amount.toBigDecimal().div(BigDecimal.fromString(getZeros(0)));
}


