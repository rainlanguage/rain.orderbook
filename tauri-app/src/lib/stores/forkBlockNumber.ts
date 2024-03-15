import { getBlockNumberFromRpc } from "$lib/services/chain";
import { fetchableIntStore } from "$lib/storesGeneric/fetchableStore";
import { get } from "svelte/store";
import { rpcUrl } from "./settings";

export const forkBlockNumber = fetchableIntStore("forkBlockNumber", async () => {
  const $rpcUrl = get(rpcUrl);
  if(!$rpcUrl) return 0;

  return getBlockNumberFromRpc($rpcUrl);
});

// When active chain updated, reset active orderbook
rpcUrl.subscribe(async ()  => {
  await rpcUrl.load();
  forkBlockNumber.fetch();
});
