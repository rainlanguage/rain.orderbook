import { Client, cacheExchange, fetchExchange } from "@urql/svelte";
import { derived, writable } from "svelte/store";

export const subgraphClientEndpoint = writable<string | null>(null);

/**
 * Set the subgraph client.
 * Once set the subgraph client will be available in the $subgraphClient store.
 * @param endpoint - The endpoint of the subgraph
 */
export const setSubgraphClient = (endpoint: string) => {
    subgraphClientEndpoint.set(endpoint);
}

export const subgraphClient = derived(subgraphClientEndpoint, ($subgraphClientEndpoint) => {
    if ($subgraphClientEndpoint) {
        return new Client({
            url: $subgraphClientEndpoint,
            exchanges: [cacheExchange, fetchExchange],
            requestPolicy: 'cache-and-network'
        });
    } else {
        return null;
    }
})