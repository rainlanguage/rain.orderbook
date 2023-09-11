import { subgraphClient } from "$lib/stores";
import { derived, writable, type Readable } from "svelte/store";
import { graphql } from "../generated";
import type { CombinedError } from "@urql/svelte";
import type { TokenVault_Filter, TokenVaultsQuery } from "$lib/gql/generated/graphql";

const tokenVaults = graphql(`query tokenVaults ($filters: TokenVault_filter) {
    tokenVaults (where: $filters) {
        vaultId
        orders {
            id
            orderHash
            orderActive
            expression
            expressionDeployer
        }
        owner {
            id 
        }
        balance
        balanceDisplay
        id
        token {
            symbol
            name
            decimals
            id
        }    
    }
  }`)

/**
 * General query for vaults
 * Optionally filter by owners, orders.
 * 
 * Returns a stores with the result of the query, the owners and orders variables and a refresh function.
 * Modifying the owners or orders stores, or calling the refresh function, will trigger a new query and update the result store.
 * @param options 
 */
export const queryTokenVaults = (options?: { owners?: string[], orders?: string[], tokens?: string[], vaultIds?: string[] }) => {
    const owners = writable(options?.owners || null)
    const orders = writable(options?.orders || null)
    const tokens = writable(options?.tokens || null)
    const vaultIds = writable(options?.vaultIds || null)
    const refreshStore = writable(1)

    const result: Readable<{ data?: TokenVaultsQuery['tokenVaults'], error?: CombinedError }> = derived(
        [subgraphClient, owners, orders, tokens, vaultIds, refreshStore],
        ([$subgraphClient, $owners, $orders, $tokens, $vaultIds, $refreshStore], set) => {
            if ($subgraphClient) {
                let filters: TokenVault_Filter = {}
                if ($owners?.length) filters.owner_in = $owners
                if ($orders?.length) filters.orders_ = { orderHash_in: $orders }
                if ($tokens?.length) filters.token_in = $tokens
                if ($vaultIds?.length) filters.vaultId_in = $vaultIds

                $subgraphClient.query(tokenVaults, { filters }).then((result) => {
                    if (result.data?.tokenVaults) {
                        set({ data: result.data.tokenVaults });
                    } else if (result.error) {
                        set({ error: result.error });
                    }
                });
            } else {
                // Set the derived store value to `null` when the `$subgraphClient` is `null`
                set({});
            }
        }
    );

    const refresh = () => {
        refreshStore.update(n => n + 1);
    }

    return { result, owners, orders, tokens, vaultIds, refresh };
}