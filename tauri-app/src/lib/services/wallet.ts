import { chainId, rpcUrl } from "$lib/stores/settings";
import { invoke } from "@tauri-apps/api";
import { get } from "svelte/store";

export const getAddressFromLedger = (derivationIndex: number): Promise<string> => invoke('get_address_from_ledger', {
  derivationIndex,
  chainId: get(chainId),
  rpcUrl: get(rpcUrl)
});
