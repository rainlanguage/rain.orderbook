import { activeChain } from "$lib/stores/settings";
import { get } from "svelte/store";

export function formatBlockExplorerTransactionUrl(txHash: string) {
  const c = get(activeChain);
  if(!c || !c.blockExplorers?.default) return;

  return `${c.blockExplorers?.default.url}/tx/${txHash}`;
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function formatEthersTransactionError(e: any) {
  if (typeof e === "object" && e?.error) {
    return `Transaction failed, error: 
    ${JSON.stringify(e.error)}`;
  }
  else if (typeof e === "object") {
    return `Transaction failed, error: 
    ${JSON.stringify(e)}`;
  }
  else if (typeof e === "string") return e;
  else if (e instanceof Error) return e.message;
  else {
    return "Transaction failed!";
  }
}