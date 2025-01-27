<script lang="ts">
	import type { Writable } from 'svelte/store';
	import type { Config } from '@wagmi/core';
	import DeploymentSteps from './DeploymentSteps.svelte';

	export let dotrain: string;
	export let key: string;
	export let name: string;
	export let description: string;
	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
</script>

<DeploymentSteps {dotrain} deployment={key} deploymentDetails={{ name, description }} {wagmiConfig} {wagmiConnected}/>
