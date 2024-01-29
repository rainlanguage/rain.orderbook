import { getAddress, isAddress } from "viem";

export function formatAddressShorthand(address: string, size=4) {
  if(!isAddress(address)) throw Error("Must be a valid address");
  const cased = getAddress(address);

  return `${cased.slice(0, size+2)}...${cased.slice(cased.length-size, cased.length)}`;
}
