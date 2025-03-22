<script context="module" lang="ts">
	export const ACCOUNT_KEY = 'account_key';
	export const USE_ACCOUNT_KEY = 'use_account_key';
	export type UseAccountResult = {
		account: Readable<string | null>;
	};
	export type UseAccount = () => UseAccountResult;
</script>

<script lang="ts">
	import { setContext } from 'svelte';
	import { readable, type Readable } from 'svelte/store';

	export let account: Readable<string | null> = readable(null);

	// Set the account address store in the context
	setContext(ACCOUNT_KEY, account);

	const useAccount: UseAccount = () => {
		return {
			account: account
		};
	};
	// Set the useAccount function in the context
	setContext(USE_ACCOUNT_KEY, useAccount);
</script>

<slot />
