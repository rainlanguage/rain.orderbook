import { subgraphClient } from "$lib/stores";
import { derived, writable, type Readable } from "svelte/store";
import { graphql } from "../generated";
import type { TakeOrderEntitiesDynamicFilterQuery, TakeOrderEntity_Filter } from "$lib/gql/generated/graphql";
import type { CombinedError } from "@urql/svelte";

const takeOrderEntitiesQuery = graphql(`query takeOrderEntitiesDynamicFilter ($filters: TakeOrderEntity_filter) {
    takeOrderEntities (where: $filters, orderBy: timestamp, orderDirection: desc) {
		id
		input
		inputDisplay
		output
		outputDisplay
		timestamp
		order {
			orderHash
			id
			owner {
				id
			}
		}
		inputToken {
			id
			name
			symbol
			decimals
		}
		outputToken {
			id
			name
			symbol
			decimals
		}
		sender {
			id
		}
		transaction {
			timestamp
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
export const queryTakeOrderEntities = (options?: { owners?: string[], orders?: string[], inputTokens?: string[], outputTokens?: string[] }) => {
	const owners = writable(options?.owners || null)
	const orders = writable(options?.orders || null)
	const inputTokens = writable(options?.inputTokens || null)
	const outputTokens = writable(options?.outputTokens || null)

	const refreshStore = writable(1)

	const result: Readable<{ data?: TakeOrderEntitiesDynamicFilterQuery['takeOrderEntities'], error?: CombinedError }> = derived(
		[subgraphClient, owners, orders, inputTokens, outputTokens, refreshStore],
		([$subgraphClient, $owners, $orders, $inputTokens, $outputTokens, $refreshStore], set) => {
			if ($subgraphClient) {
				let filters: TakeOrderEntity_Filter = {}
				if ($owners?.length) filters.order_ = { owner_in: $owners }
				if ($orders?.length) filters.order_ = { ...filters.order_, orderHash_in: $orders }
				if ($inputTokens?.length) filters.inputToken_in = $inputTokens
				// const orFilters = []
				// if ($inputTokens?.length) orFilters.push({ inputToken_in: $inputTokens })
				// if ($outputTokens?.length) orFilters.push({ outputToken_in: $outputTokens })
				// if (orFilters.length) filters.or = orFilters
				// if ($inputTokens?.length) filters.or = [{ inputToken_in: $inputTokens }]
				// if ($outputTokens?.length) filters.or.push({ outputToken_in: $outputTokens })
				if ($outputTokens?.length) filters.outputToken_in = $outputTokens
				$subgraphClient.query(takeOrderEntitiesQuery, { filters }).toPromise().then((result) => {
					if (result.data?.takeOrderEntities) {
						set({ data: result.data.takeOrderEntities });
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

	return { result, owners, orders, inputTokens, outputTokens, refresh };
}