<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, PageHeader } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal } from '$lib/services/modal';
	import { handleUpdateGuiState } from '$lib/services/handleUpdateGuiState';
	const { dotrain, deployment } = $page.data;

	if (!dotrain || !deployment) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	const stateFromUrl = $page.url.searchParams.get('state') || '';
</script>

<PageHeader title={$page.data.deployment.name || 'Deploy'} pathname={$page.url.pathname}
></PageHeader>

{#if !dotrain || !deployment}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else}
	<DeploymentSteps
		{dotrain}
		{deployment}
		{wagmiConfig}
		wagmiConnected={connected}
		{appKitModal}
		{handleDeployModal}
		{stateFromUrl}
		{handleUpdateGuiState}
	/>
{/if}
