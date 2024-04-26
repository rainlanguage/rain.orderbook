import { chainId, rpcUrl } from "$lib/stores/settings";
import { invoke } from "@tauri-apps/api";
import { get } from "svelte/store";

export const getAddressFromLedger = (derivationIndex: number): Promise<string> => invoke('get_address_from_ledger', {
  derivationIndex,
  chainId: get(chainId),
  rpcUrl: get(rpcUrl)
});

export const getBalanceFromLedger = async (derivationIndex: number): Promise<unknown> => {
  try {
    const balance = await invoke('get_balance_from_ledger', {
      derivationIndex,
      chainId: get(chainId),
      rpcUrl: get(rpcUrl)
    });
    return balance;
  } catch (error) {
    //console.error('Error in getBalanceFromLedger:', error, 'derivationIndex', derivationIndex);
  }
};

export const getBalanceFromWallet = async (address: string): Promise<string | undefined> => {
  try {
    const _rpcUrl = get(rpcUrl);
    if (_rpcUrl) {
      const balance = await invoke('get_balance_from_wallet', {
        address,
        rpcUrl: _rpcUrl
      });
      console.log('balance', balance);
      return balance as string;
    }
    return undefined;
  } catch (error) {
    console.error({'Error in getBalance': error, 'address': address});
  }
}
