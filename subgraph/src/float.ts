import { Address, Bytes, dataSource } from "@graphprotocol/graph-ts";
import { DecimalFloat } from "../generated/OrderBook/DecimalFloat";

export type Float = Bytes;

export const FALLBACK_DECIMAL_FLOAT_ADDRESS = Address.fromString(
  "0x0000000000000000000000000000000000000001"
);

export function getCalculator(): DecimalFloat {
  return DecimalFloat.bind(getDecimalFloatAddress());
}

function getDecimalFloatAddress(): Address {
  let network = dataSource.network();
  if (network == "flare") {
    return Address.fromString("0xDbcb964760d021e18A31C9A731d8589c361E0E20");
  } else if (network == "base") {
    return Address.fromString("0x83e4c7732e715b5E7310796A4A2a21d89f3FB59A");
  } else if (network == "bsc") {
    return Address.fromString("0xDbcb964760d021e18A31C9A731d8589c361E0E20");
  } else if (network == "arbitrum-one") {
    return Address.fromString("0x3ae05C7A18e003299D3db30fB3b2caA67a35a4dE");
  } else if (network == "matic") {
    return Address.fromString("0x83e4c7732e715b5E7310796A4A2a21d89f3FB59A");
  } else if (network == "linea") {
    return Address.fromString("0x83e4c7732e715b5E7310796A4A2a21d89f3FB59A");
  } else if (network == "mainnet") {
    return Address.fromString("0x83e4c7732e715b5E7310796A4A2a21d89f3FB59A");
  }

  return FALLBACK_DECIMAL_FLOAT_ADDRESS;
}
