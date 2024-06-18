import { Bytes, ethereum } from "@graphprotocol/graph-ts";

export function eventId(event: ethereum.Event): Bytes {
  return event.transaction.hash.concatI32(event.logIndex.toI32());
}
