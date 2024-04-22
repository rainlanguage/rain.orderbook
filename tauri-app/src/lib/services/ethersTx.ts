import { toasts } from "$lib/stores/toasts";
import { get } from "@square/svelte-store";
import { BigNumber, ethers } from "ethers";
import { walletconnectAccount, walletconnectProvider } from "$lib/stores/walletconnect";

export async function ethersExecute(calldata: Uint8Array, to: string): Promise<ethers.providers.TransactionResponse> {
  if (!walletconnectProvider || !get(walletconnectAccount)) {
    toasts.error("user not connected");
    return Promise.reject("user not connected");
  }
  else {
    const ethersProvider = new ethers.providers.Web3Provider(walletconnectProvider);
    const signer = ethersProvider.getSigner();
    const rawtx = {
      data: calldata,
      to,
    };
    try {
      return signer.sendTransaction(rawtx);
    } catch (error) {
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      (<any>error).rawtx = rawtx;
      throw error;
    }
  }
}

const abi = [
  "function allowance(address owner, address spender) view returns (uint256)"
];

export async function checkAllowance(tokenAddress: string, spender: string): Promise<BigNumber> {
  if (!walletconnectProvider) {
    toasts.error("user not connected");
    return Promise.reject("user not connected");
  }
  else {
    const ethersProvider = new ethers.providers.Web3Provider(walletconnectProvider);
    const signer = ethersProvider.getSigner();
    const contract = new ethers.Contract(tokenAddress, abi, signer);
    return contract.allowance(await signer.getAddress(), spender) as BigNumber;
  }
}