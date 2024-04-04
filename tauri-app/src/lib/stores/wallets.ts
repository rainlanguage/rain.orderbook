import { cachedWritableInt } from "$lib/storesGeneric/cachedWritableStore";
import { validatedStringStore } from "$lib/storesGeneric/settingStore";
import { derived } from "svelte/store";
import { isAddress } from "viem";
import { walletconnectAccount } from "$lib/stores/walletconnect"

export const ledgerWalletAddress = validatedStringStore("settings.walletAddress", "", isAddress);
export const ledgerWalletDerivationIndex = cachedWritableInt("settings.walletDerivationIndex", 0);
export const walletAddressMatchesOrBlank = derived(
  [ledgerWalletAddress, walletconnectAccount],
  ([$walletAddress, $walletconnectAccount]) => {
    return (otherAddress: string) => {
      const otherAddressLowercase = otherAddress.toLowerCase();
      return $walletAddress.value.toLowerCase() === otherAddressLowercase
        || $walletconnectAccount?.toLowerCase() === otherAddressLowercase
    }
  }
);
