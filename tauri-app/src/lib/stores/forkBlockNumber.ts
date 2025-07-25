import { getBlockNumberFromRpc } from '$lib/services/chain';
import { fetchableIntStore } from '$lib/storesGeneric/fetchableStore';

export const forkBlockNumber = fetchableIntStore('forkBlockNumber', async (rpcs) => {
  return getBlockNumberFromRpc(rpcs as string[]);
});
