<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, GuiProvider } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal, signerAddress } from '$lib/stores/wagmi';
	import { handleDeployModal, handleDisclaimerModal } from '$lib/services/modal';
	import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
	import { onMount } from 'svelte';
	import { handleGuiInitialization } from '$lib/services/handleGuiInitialization';
	import { REGISTRY_URL } from '$lib/constants';

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
		if (dotrain && deployment) {
			const customRegistryUrl = $page.url.searchParams?.get('registry');
			if (!customRegistryUrl) {
				const registryUrl = REGISTRY_URL;
				window.history.pushState({}, '', window.location.pathname + '?registry=' + registryUrl);
			}
			const { gui: initializedGui, error } = await handleGuiInitialization(
				dotrain,
				deployment.key,
				stateFromUrl
			);
			gui = initializedGui;
			getGuiError = error;
		}
	});
</script>

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
