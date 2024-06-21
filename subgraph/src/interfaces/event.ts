import { Bytes, ethereum } from "@graphprotocol/graph-ts";

export function eventId(event: ethereum.Event): Bytes {
  return event.transaction.hash.concat(
    Bytes.fromByteArray(Bytes.fromBigInt(event.logIndex))
  );
}
