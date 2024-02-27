import { activeChain } from "$lib/stores/settings";
import { get } from "svelte/store";

export function formatBlockExplorerTransactionUrl(txHash: string) {
  const c = get(activeChain);
  if(!c || !c.blockExplorers?.default) return;

  return `${c.blockExplorers?.default.url}/tx/${txHash}`;
}