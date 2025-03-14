<script lang="ts">
	import { setContext } from 'svelte';
	import { WAGMI_CONTEXT_KEY } from '@rainlanguage/ui-components';
	import * as wagmiStores from '$lib/stores/wagmi';
	import { browser } from '$app/environment';
	import { onMount } from 'svelte';

	export let appName: string;
	export let projectId: string;
	export let chains: any[];
	export let connectors: any[];

	setContext(WAGMI_CONTEXT_KEY, wagmiStores);

	onMount(() => {
		if (browser && window.navigator) {
			const init = wagmiStores.defaultConfig({
				appName,
				connectors,
				chains,
				projectId
			});

			init.init();
		}
	});
</script>

<slot />
