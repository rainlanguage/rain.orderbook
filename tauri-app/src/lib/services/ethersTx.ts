import { toasts } from '$lib/stores/toasts';
import { get } from '@square/svelte-store';
import { BigNumber, ethers } from 'ethers';
import { walletconnectAccount, walletconnectProvider } from '$lib/stores/walletconnect';

export async function ethersExecute(
  calldata: Uint8Array,
  to: string,
): Promise<ethers.providers.TransactionResponse> {
  if (!walletconnectProvider || !get(walletconnectAccount)) {
    toasts.error('user not connected');
    return Promise.reject('user not connected');
  } else {
    const ethersProvider = new ethers.providers.Web3Provider(walletconnectProvider);
    const signer = ethersProvider.getSigner();
    const rawtx = {
      data: calldata,
      to,
    };
    return signer.sendTransaction(rawtx);
  }
}

const allowanceAbi = ['function allowance(address owner, address spender) view returns (uint256)'];
const balanceOfAbi = ['function balanceOf(address account) view returns (uint256)'];

export async function checkAllowance(tokenAddress: string, spender: string): Promise<BigNumber> {
  if (!walletconnectProvider) {
    toasts.error('user not connected');
    return Promise.reject('user not connected');
  } else {
    const ethersProvider = new ethers.providers.Web3Provider(walletconnectProvider);
    const signer = ethersProvider.getSigner();
    const contract = new ethers.Contract(tokenAddress, allowanceAbi, signer);
    return contract.allowance(await signer.getAddress(), spender) as BigNumber;
  }
}

export async function checkERC20Balance(tokenAddress: string): Promise<BigNumber> {
  if (!walletconnectProvider) {
    toasts.error('user not connected');
    return Promise.reject('user not connected');
  } else {
    const ethersProvider = new ethers.providers.Web3Provider(walletconnectProvider);
    const signer = ethersProvider.getSigner();
    console.log('signer', signer);
    const contract = new ethers.Contract(tokenAddress, balanceOfAbi, signer);
    return contract.balanceOf(await signer.getAddress()) as BigNumber;
  }
}
