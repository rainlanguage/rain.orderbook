<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, PageHeader } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal, handleDisclaimerModal } from '$lib/services/modal';
	import { handleUpdateGuiState } from '$lib/services/handleUpdateGuiState';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import { onMount } from 'svelte';

	const { settings } = $page.data.stores;
	const { dotrain, deployment, strategyDetail } = $page.data;
	const stateFromUrl = $page.url.searchParams?.get('state') || '';

	let gui: DotrainOrderGui | null = null;
	let getGuiError: string | null = null;

	if (!dotrain || !deployment) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	onMount(async () => {
		try {
			if (stateFromUrl) {
				return (gui = await DotrainOrderGui.deserializeState(
					dotrain,
					$page.url.searchParams.get('state') || ''
				));
			} else {
				gui = await DotrainOrderGui.chooseDeployment(dotrain, deployment.key);
			}
		} catch (err) {
			getGuiError = 'Could not get deployment form.';
		}
	});
</script>

<PageHeader title={$page.data.deployment.name || 'Deploy'} pathname={$page.url.pathname}
></PageHeader>

{#if !dotrain || !deployment}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if gui}
	<DeploymentSteps
		{strategyDetail}
		{gui}
		{dotrain}
		{deployment}
		{wagmiConfig}
		wagmiConnected={connected}
		{appKitModal}
		{handleDeployModal}
		{settings}
		{handleUpdateGuiState}
		{handleDisclaimerModal}
	/>
{:else if getGuiError}
	<div>
		{getGuiError}
	</div>
{/if}
