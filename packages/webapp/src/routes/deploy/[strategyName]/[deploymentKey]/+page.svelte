<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps, PageHeader } from '@rainlanguage/ui-components';
	import { wagmiConfig, connected, appKitModal } from '$lib/stores/wagmi';
	import { handleDeployModal } from '$lib/services/modal';
	import { handleUpdateGuiState } from '$lib/services/handleUpdateGuiState';
	import { Button, Textarea, Toggle } from 'flowbite-svelte';
	import { rawDotrain } from '$lib/stores/raw-dotrain';
	const { dotrain, deployment } = $page.data;

	if (!dotrain || !deployment) {
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	const stateFromUrl = $page.url.searchParams.get('state') || '';

	let advancedMode = false;
	let inputDotrain = '';

	const loadRawStrategy = () => {
		if (inputDotrain.trim()) {
			$rawDotrain = inputDotrain;
			inputDotrain = '';
		}
	};
</script>

<PageHeader title={$page.data.deployment.name || 'Deploy'} pathname={$page.url.pathname}>
	<svelte:fragment slot="actions">
		<Toggle on:change={() => (advancedMode = !advancedMode)}>
			{'Advanced Mode'}
		</Toggle></svelte:fragment
	>
</PageHeader>

<div class="flex items-start justify-end gap-4">
	{#if advancedMode}
		<div class="mb-12 flex w-2/3 flex-col items-start gap-4">
			<div class="flex w-full items-start gap-4">
				<Textarea id="textarea-id" placeholder="Raw strategy" rows="8" bind:value={inputDotrain} />
				<Button class="text-nowrap" on:click={loadRawStrategy}>Load Raw Strategy</Button>
			</div>
		</div>
	{/if}
</div>

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
