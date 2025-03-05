<script lang="ts">
	import { setWagmiContext, setAppKitContext, setIsLoadingContext } from './context';
	import { loading, wagmiConfig, appKitModal } from './wagmiStores';
	import { get } from 'svelte/store';
	import type { Config } from '@wagmi/core';
	import type { AppKit } from '@reown/appkit';

	// Use the stores directly from wagmi.js, but allow for overrides
	export let config: Config = get(wagmiConfig);
	export let appKit: AppKit = get(appKitModal);

	// Set the context values so they can be accessed by descendants
	setWagmiContext(config);
	setAppKitContext(appKit);
	setIsLoadingContext(loading);

	// Update context if props change
	$: if (config !== get(wagmiConfig)) {
		setWagmiContext(config);
	}

	$: if (appKit !== get(appKitModal)) {
		setAppKitContext(appKit);
	}
</script>

<slot />
