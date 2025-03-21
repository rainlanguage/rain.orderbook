<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, GuiProvider, PageHeader } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal, signerAddress } from '$lib/stores/wagmi';
	import { handleDeployModal, handleDisclaimerModal } from '$lib/services/modal';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import { onMount } from 'svelte';
	import { handleGuiInitialization } from '$lib/services/handleGuiInitialization';

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
		const { gui: initializedGui, error } = await handleGuiInitialization(
			dotrain,
			deployment.key,
			stateFromUrl
		);
		gui = initializedGui;
		getGuiError = error;
	});
</script>

<PageHeader title={$page.data.deployment.name || 'Deploy'} pathname={$page.url.pathname}
></PageHeader>

{#if !dotrain || !deployment}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if gui}
	<GuiProvider {gui}>
		<DeploymentSteps
			{strategyDetail}
			{dotrain}
			{deployment}
			{wagmiConfig}
			wagmiConnected={connected}
			{appKitModal}
			{handleDeployModal}
			{settings}
			{handleDisclaimerModal}
			{signerAddress}
		/>
	</GuiProvider>
{:else if getGuiError}
	<div>
		{getGuiError}
	</div>
{/if}
