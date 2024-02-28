import { cachedWritableInt } from "$lib/storesGeneric/cachedWritableStore";
import { validatedStringStore } from "$lib/storesGeneric/settingStore";
import { derived } from "svelte/store";
import { isAddress } from "viem";

export const walletAddress = validatedStringStore("settings.walletAddress", "", isAddress);
export const walletDerivationIndex = cachedWritableInt("settings.walletDerivationIndex", 0);
export const walletAddressMatchesOrBlank = derived(walletAddress, $walletAddress => {
  return (otherAddress: string) => $walletAddress.value === otherAddress || $walletAddress.value === "";
});
