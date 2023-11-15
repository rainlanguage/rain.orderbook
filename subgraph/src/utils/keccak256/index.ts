import { Bytes, crypto } from "@graphprotocol/graph-ts";

export function getKeccak256FromBytes(data_: Bytes): Bytes {
  return Bytes.fromByteArray(crypto.keccak256(Bytes.fromByteArray(data_)));
}
