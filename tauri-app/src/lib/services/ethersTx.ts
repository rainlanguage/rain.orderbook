import { walletconnectAccount, walletconnectIsConnected, walletconnectModal } from "$lib/stores/settings";
import { toasts } from "$lib/stores/toasts";
import { get } from "@square/svelte-store";
import { BigNumber, ethers } from "ethers";

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

const abi = [
  "function allowance(address owner, address spender) view returns (uint256)"
];

export async function checkAllowance(amount: bigint, tokenAddress: string, spender: string): Promise<boolean> {
  const walletProvider = get(walletconnectModal)?.getWalletProvider();
  if (!walletProvider) {
    toasts.error("user not connected");
    return Promise.reject("user not connected");
  }
  else {
    const ethersProvider = new ethers.providers.Web3Provider(walletProvider);
    const signer = ethersProvider.getSigner();
    const contract = new ethers.Contract(tokenAddress, abi, signer);
    const allowance = await contract.allowance(await signer.getAddress(), spender) as BigNumber;
    return allowance.gte(amount);
  }
}