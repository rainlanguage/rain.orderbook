import { getBlockNumberFromRpc } from "$lib/services/chain";
import { fetchableIntStore } from "$lib/storesGeneric/fetchableStore";
import { get } from "svelte/store";
import { rpcUrl } from "./settings";

export const forkBlockNumber = fetchableIntStore("forkBlockNumber", async () =>  await getBlockNumberFromRpc(get(rpcUrl)));
