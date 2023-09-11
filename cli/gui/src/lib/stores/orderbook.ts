import { derived, writable, type Readable } from "svelte/store";
import { IOrderBookV2 } from '$lib/abi/IOrderBookV2'
import { makeContractStore, type WagmiContract } from 'svelte-wagmi-stores'

export const orderbookAddress = writable<`0x${string}`>();

/**
 * Set the address of the orderbook contract.
 * Once set it will be available in the $orderbook store.
 * @param address - Address of the orderbook contract
 */
export const setOrderbookAddress = (address: `0x${string}`) => orderbookAddress.set(address)

/**
 * Store for the orderbook contract.
 * This is a derived store that is created from the orderbookAddress store.
 * It will be updated whenever the orderbookAddress store is updated.
 * @returns - Readable WagmiContract store for the orderbook contract
 */
export const orderbook: Readable<WagmiContract<typeof IOrderBookV2>> = derived(orderbookAddress, ($orderbookAddress, set) => {
    // This is the derived store returned from makeContractStore
    const innerStore = makeContractStore(IOrderBookV2, $orderbookAddress);

    // Subscribe to the innerStore and set the value for the outer derived store
    const unsubscribe = innerStore.subscribe(value => {
        set(value);
    });

    return () => {
        unsubscribe(); // Cleanup when the outer derived store is no longer in use
    };
})

