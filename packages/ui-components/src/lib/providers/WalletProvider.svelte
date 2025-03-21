<script context="module" lang="ts">
	export const SIGNER_ADDRESS_KEY = 'signer_address_key';
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

	setContext(SIGNER_ADDRESS_KEY, account);

	const useAccount: UseAccount = () => {
		return {
			account: account
		};
	};

	setContext(USE_ACCOUNT_KEY, useAccount);
</script>

<slot />
