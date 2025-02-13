<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentPage } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal } from '$lib/services/modal';
	import { handleUpdateGuiState } from '$lib/services/handleUpdateGuiState';
	const { dotrain, key, name, description } = $page.data;
	const { settings } = $page.data.stores;

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
		{settings}
		{handleUpdateGuiState}
	/>
{/if}
