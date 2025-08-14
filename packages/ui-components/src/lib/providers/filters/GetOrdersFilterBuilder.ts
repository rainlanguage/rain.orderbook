import type { Address, Hex, GetOrdersFilters } from '@rainlanguage/orderbook';

/**
 * Builder class for creating and updating orders filters with a fluent API.
 * Provides a clean, type-safe way to construct filter objects.
 *
 * @example
 * ```typescript
 * const filters = new GetOrdersFilterBuilder(currentFilters)
 *   .setOwners([address1, address2])
 *   .setActive(true)
 *   .setOrderHash("0x123...")
 *   .setTokens([tokenAddress])
 *   .build();
 * ```
 */
export class GetOrdersFilterBuilder {
	private filters: GetOrdersFilters;

	constructor(currentFilters: GetOrdersFilters) {
		// Deep clone to avoid mutations
		this.filters = {
			owners: [...currentFilters.owners],
			active: currentFilters.active,
			orderHash: currentFilters.orderHash,
			tokens: currentFilters.tokens ? [...currentFilters.tokens] : currentFilters.tokens
		};
	}

	/**
	 * Set the order owners to filter by.
	 * @param owners Array of owner addresses
	 * @returns This builder instance for chaining
	 */
	setOwners(owners: Address[]) {
		this.filters.owners = [...owners];
		return this;
	}

	/**
	 * Set whether to filter by active orders.
	 * @param active Whether to filter only active orders, or undefined to clear filter
	 * @returns This builder instance for chaining
	 */
	setActive(active?: boolean) {
		this.filters.active = active;
		return this;
	}

	/**
	 * Set the order hash to filter by.
	 * @param orderHash Specific order hash to filter by, or undefined to clear filter
	 * @returns This builder instance for chaining
	 */
	setOrderHash(orderHash?: Hex) {
		this.filters.orderHash = orderHash;
		return this;
	}

	/**
	 * Set the token addresses to filter by.
	 * @param tokens Array of token addresses, or undefined to clear filter
	 * @returns This builder instance for chaining
	 */
	setTokens(tokens?: Address[]) {
		this.filters.tokens = tokens ? [...tokens] : undefined;
		return this;
	}

	/**
	 * Build the final filter object.
	 * @returns The constructed GetOrdersFilters object
	 */
	build(): GetOrdersFilters {
		return {
			owners: [...this.filters.owners],
			active: this.filters.active,
			orderHash: this.filters.orderHash,
			tokens: this.filters.tokens ? [...this.filters.tokens] : this.filters.tokens
		};
	}
}
