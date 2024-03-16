import { walletconnectAccount, walletconnectIsConnected, walletconnectModal } from "$lib/stores/settings";
import { toasts } from "$lib/stores/toasts";
import { get } from "@square/svelte-store";
import { ethers } from "ethers";

export async function ethersExecute(calldata: Uint8Array, to: string): Promise<ethers.providers.TransactionResponse> {
  const walletProvider = get(walletconnectModal)?.getWalletProvider();
  if (!walletProvider || !get(walletconnectIsConnected) || !get(walletconnectAccount)) {
    toasts.error("user not connected");
    return Promise.reject("user not connected");
  }
  else {
    const ethersProvider = new ethers.providers.Web3Provider(walletProvider);
    const signer = ethersProvider.getSigner();
    const rawtx = {
      data: calldata,
      to,
    };
    return signer.sendTransaction(rawtx);
  }
}