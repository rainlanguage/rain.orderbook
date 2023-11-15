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

export function toDisplayWithDecimals(
  amount: BigInt,
  decimals: i32
): BigDecimal {
  let denominator = BigInt.fromString(getZeros(decimals));
  return amount.toBigDecimal().div(denominator.toBigDecimal());
}

export function gcd(a: BigInt, b: BigInt): BigInt {
  if (b.equals(BigInt.zero())) {
    return a;
  } else {
    return gcd(b, a.mod(b));
  }
}

export function BDtoBIMultiplier(n1: BigDecimal, n2: BigDecimal): BigInt {
  let n1_split = n1.toString().split(".");
  let n1_decimals = n1_split.length == 1 ? 0 : n1_split[1].length;

  let n2_split = n2.toString().split(".");
  let n2_decimals = n2_split.length == 1 ? 0 : n2_split[1].length;

  let number: BigDecimal;
  if (n1_decimals > n2_decimals) {
    number = n1;
  } else {
    number = n2;
  }
  let location = number.toString().indexOf(".");
  let len = number.toString().slice(location + 1).length;
  return BigInt.fromString(getZeros(len));
}
