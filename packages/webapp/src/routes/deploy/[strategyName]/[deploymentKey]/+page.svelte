<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentPage } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal } from '$lib/services/modal';
	const { dotrain, key, name, description, settings } = $page.data;
	const { subgraphUrl } = settings;

	if (!dotrain || !key) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}
</script>

{#if !dotrain || !key}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else}
	<DeploymentPage
		{dotrain}
		{key}
		{name}
		{description}
		{wagmiConfig}
		wagmiConnected={connected}
		{appKitModal}
		{handleDeployModal}
		{subgraphUrl}
	/>
{/if}
