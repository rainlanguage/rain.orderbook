import type { Address, GetVaultsFilters } from '@rainlanguage/orderbook';

/**
 * Builder class for creating and updating vault filters with a fluent API.
 * Provides a clean, type-safe way to construct filter objects.
 *
 * @example
 * ```typescript
 * const filters = new GetVaultsFilterBuilder(currentFilters)
 *   .setOwners([address1, address2])
 *   .setHideZeroBalance(true)
 *   .setChainIds([1, 137])
 *   .build();
 * ```
 */
export class GetVaultsFilterBuilder {
	private filters: GetVaultsFilters;

	constructor(currentFilters: GetVaultsFilters) {
		// Deep clone to avoid mutations
		this.filters = {
			owners: [...currentFilters.owners],
			hideZeroBalance: currentFilters.hideZeroBalance,
			tokens: currentFilters.tokens ? [...currentFilters.tokens] : currentFilters.tokens,
			chainIds: currentFilters.chainIds ? [...currentFilters.chainIds] : currentFilters.chainIds
		};
	}

	/**
	 * Set the vault owners to filter by.
	 * @param owners Array of owner addresses
	 * @returns This builder instance for chaining
	 */
	setOwners(owners: Address[]) {
		this.filters.owners = [...owners];
		return this;
	}

	/**
	 * Set whether to hide zero balance vaults.
	 * @param hide Whether to hide zero balance vaults
	 * @returns This builder instance for chaining
	 */
	setHideZeroBalance(hide: boolean) {
		this.filters.hideZeroBalance = hide;
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
	 * Set the chain IDs to filter by.
	 * @param chainIds Array of chain IDs, or undefined to clear filter
	 * @returns This builder instance for chaining
	 */
	setChainIds(chainIds?: number[]) {
		this.filters.chainIds = chainIds ? [...chainIds] : undefined;
		return this;
	}

	/**
	 * Build the final filter object.
	 * @returns The constructed GetVaultsFilters object
	 */
	build(): GetVaultsFilters {
		return {
			owners: [...this.filters.owners],
			hideZeroBalance: this.filters.hideZeroBalance,
			tokens: this.filters.tokens ? [...this.filters.tokens] : this.filters.tokens,
			chainIds: this.filters.chainIds ? [...this.filters.chainIds] : this.filters.chainIds
		};
	}
}
