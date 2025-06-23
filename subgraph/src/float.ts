import { Address, Bytes, dataSource } from "@graphprotocol/graph-ts";
import { DecimalFloat } from "../generated/OrderBook/DecimalFloat";

export type Float = Bytes;

export function getCalculator(): DecimalFloat {
  return DecimalFloat.bind(getDecimalFloatAddress());
}

function getDecimalFloatAddress(): Address {
  let network = dataSource.network();
  if (network == "flare") {
    return Address.fromString("0x0000000000000000000000000000000000000001");
  } else if (network == "base") {
    return Address.fromString("0x0000000000000000000000000000000000000002");
  } else if (network == "bsc") {
    return Address.fromString("0x0000000000000000000000000000000000000003");
  } else if (network == "arbitrum-one") {
    return Address.fromString("0x3ae05C7A18e003299D3db30fB3b2caA67a35a4dE");
  } else if (network == "matic") {
    return Address.fromString("0x0000000000000000000000000000000000000005");
  } else if (network == "linea") {
    return Address.fromString("0x0000000000000000000000000000000000000006");
  } else if (network == "mainnet") {
    return Address.fromString("0x0000000000000000000000000000000000000007");
  }

  return Address.fromString("0x0000000000000000000000000000000000000001");
}
