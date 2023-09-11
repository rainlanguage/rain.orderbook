import { IERC20 } from '$lib/abi/IERC20'
import { makeContractStore } from 'svelte-wagmi-stores'
import type { Address } from 'viem'

export type TokenStore = ReturnType<typeof makeContractStore<typeof IERC20>>

export const makeTokenStore = (address: Address): TokenStore => {
    return makeContractStore(IERC20, address);
}