import { graphql } from "$lib/gql/generated";
import type { Order_Filter, OrdersQueryQuery } from "$lib/gql/generated/graphql";
import { subgraphClient } from "$lib/stores";
import type { CombinedError } from "@urql/svelte";
import { writable, type Readable, derived } from "svelte/store";

const ordersQuery = graphql(`query ordersQuery ($filters: Order_filter) {
    orders (where: $filters, orderBy: timestamp, orderDirection: desc){
        id
        orderHash
        owner { id }
        orderJSONString
        orderActive
        timestamp
        expression
        validInputs {
            vaultId
            token {
                id
            }
            tokenVault {
                id
                balance
                balanceDisplay
                token {
                    name
                    decimals
                    symbol
                }
            }
        }
        validOutputs {
            vaultId
            token {
                id
            }
            tokenVault {
                id
                balance
                balanceDisplay
                token {
                    name
                    decimals
                    symbol
                }
            }
        }
        takeOrders {
            outputIOIndex
            inputIOIndex
            input
            output
            inputDisplay
            outputDisplay
            inputToken {
                decimals
                id
                name
                symbol
            }
            outputToken {
                decimals
                id
                name
                symbol
            }
            sender {
                id
            }
            timestamp
            transaction {
                blockNumber
                timestamp
                id
            }
            id
        }
    }
  }`)


/**
 * General query for take order entities.
 * Optionally filter by owners, orders.
 * 
 * Returns a stores with the result of the query, the owners and orders variables and a refresh function.
 * Modifying the owners or orders stores, or calling the refresh function, will trigger a new query and update the result store.
 * @param options 
 */
export const queryOrders = (options?: { owners?: string[], orders?: string[], validInputs?: string[], validOutputs?: string[] }) => {
    const owners = writable(options?.owners || null)
    const orders = writable(options?.orders || null)
    const validInputs = writable(options?.validInputs || null)
    const validOutputs = writable(options?.validOutputs || null)
    const refreshStore = writable(1)

    const result: Readable<{ data?: OrdersQueryQuery['orders'], error?: CombinedError }> = derived(
        [subgraphClient, owners, validInputs, validOutputs, orders, refreshStore],
        ([$subgraphClient, $owners, $validInputs, $validOutputs, $orders, $refreshStore], set) => {
            if ($subgraphClient) {
                let filters: Order_Filter = {}
                if ($owners?.length) filters.owner_in = $owners
                if ($validInputs?.length) filters.validInputs_ = { token_in: $validInputs }
                if ($validOutputs?.length) filters.validOutputs_ = { token_in: $validOutputs }
                if ($orders?.length) filters.id_in = $orders

                $subgraphClient.query(ordersQuery, { filters }).then((result) => {
                    if (result.data?.orders) {
                        set({ data: result.data.orders });
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

    return { result, owners, orders, validInputs, validOutputs, refresh };
}