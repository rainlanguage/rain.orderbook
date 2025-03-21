<script context="module" lang="ts">
	export const SIGNER_ADDRESS_KEY = 'signer_address_key';
	export const USE_ACCOUNT_KEY = 'use_account_key';

	export type UseAccountResult = {
		signerAddress: Readable<string | null>;
		addressMatchesSigner: (address: string) => boolean;
	};

	export type UseAccount = () => UseAccountResult;
</script>

<script lang="ts">
	import { setContext } from 'svelte';
	import { get, readable, type Readable } from 'svelte/store';

	export let signerAddress: Readable<string | null> = readable(null);

	setContext(SIGNER_ADDRESS_KEY, signerAddress);

	const useAccount: UseAccount = () => {
		return {
			signerAddress: signerAddress,
			addressMatchesSigner: (address: string) => {
				const currentSigner = get(signerAddress);
				return address.toLowerCase() === currentSigner?.toLowerCase();
			}
		};
	};

	setContext(USE_ACCOUNT_KEY, useAccount);
</script>

<slot />
