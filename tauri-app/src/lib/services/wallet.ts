import { chainId } from "$lib/stores/settings";
import { invoke } from "@tauri-apps/api";
import { get } from "svelte/store";

export const getAddressFromLedger = (derivationIndex: number): Promise<`0x${string}`> => invoke('get_address_from_ledger', {
  chainId: get(chainId),
  derivationIndex,
});
