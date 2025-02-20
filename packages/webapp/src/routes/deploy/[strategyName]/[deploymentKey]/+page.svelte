<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, PageHeader } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal, handleDisclaimerModal } from '$lib/services/modal';
	import { pushGuiStateToUrlHistory } from '$lib/services/handleUpdateGuiState';
	const { settings } = $page.data.stores;
	const { dotrain, deployment, strategyDetail } = $page.data;

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
		{strategyDetail}
		{dotrain}
		{deployment}
		{wagmiConfig}
		wagmiConnected={connected}
		{appKitModal}
		{handleDeployModal}
		{settings}
		{stateFromUrl}
		{pushGuiStateToUrlHistory}
		{handleDisclaimerModal}
	/>
{/if}
