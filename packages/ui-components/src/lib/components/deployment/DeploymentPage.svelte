<script lang="ts">
	import type { Writable } from 'svelte/store';
	import DeploymentSteps from './DeploymentSteps.svelte';
	import type { Config } from 'wagmi';
	import type { AppKit } from '@reown/appkit';

	export let dotrain: string;
	export let key: string;
	export let name: string;
	export let description: string;
	export let wagmiConfig: Writable<Config | undefined>;
	export let wagmiConnected: Writable<boolean>;
	export let appKitModal: Writable<AppKit>;
export let transactionStore;
</script>

<DeploymentSteps
	{dotrain}
	deployment={key}
	deploymentDetails={{ name, description }}
	{wagmiConfig}
	{wagmiConnected}
	{appKitModal}
/>
