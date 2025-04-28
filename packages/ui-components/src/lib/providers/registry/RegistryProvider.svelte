<script lang="ts">
	import { setRegistryContext } from './context';
	import type { RegistryStore } from '$lib/types/registry';

	export let registryManagerStore: RegistryStore;

	setRegistryContext(registryManagerStore);
</script>

<slot />
