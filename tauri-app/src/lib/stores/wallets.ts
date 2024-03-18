import { cachedWritableInt } from "$lib/storesGeneric/cachedWritableStore";
import { validatedStringStore } from "$lib/storesGeneric/settingStore";
import { derived } from "svelte/store";
import { isAddress } from "viem";
import { walletconnectAccount } from "$lib/stores/walletconnect"

export const walletAddress = validatedStringStore("settings.walletAddress", "", isAddress);
export const walletDerivationIndex = cachedWritableInt("settings.walletDerivationIndex", 0);
export const walletAddressMatchesOrBlank = derived(
  [walletAddress, walletconnectAccount],
  ([$walletAddress, $walletconnectAccount]) => {
    return (otherAddress: string) => {
      const otherAddressLowercase = otherAddress.toLowerCase();
      return $walletAddress.value.toLowerCase() === otherAddressLowercase
        || $walletconnectAccount?.toLowerCase() === otherAddressLowercase
        || $walletAddress.value === "";
    }
  }
);
