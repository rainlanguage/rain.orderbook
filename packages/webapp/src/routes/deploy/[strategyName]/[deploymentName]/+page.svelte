<script lang="ts">
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { DeploymentSteps } from '@rainlanguage/ui-components';
	import { DotrainOrderGui, type NameAndDescription } from '@rainlanguage/orderbook/js_api';
	import { Spinner } from 'flowbite-svelte';

	const { dotrain, deploymentName } = $page.data;

	let deploymentDetails: NameAndDescription;
	let isLoading = true;
	let deploymentNotFound: boolean;

	async function getDeploymentDetails(dotrain: string, selectedDeployment: string) {
		try {
			deploymentDetails = await DotrainOrderGui.getDeploymentDetails(dotrain, selectedDeployment);
			isLoading = false;
		} catch (error) {
			isLoading = false;
			console.log(error);
		}
	}

	if (!dotrain || !deploymentName) {
		deploymentNotFound = true;
		isLoading = false;
		setTimeout(() => {
			goto('/deploy');
		}, 5000);
	}

	if (dotrain && deploymentName) {
		getDeploymentDetails(dotrain, deploymentName);
	}
</script>

{#if isLoading}
	<Spinner />
{:else if deploymentNotFound}
	<div>Deployment not found. Redirecting to deployments page...</div>
{:else if dotrain && deploymentName && deploymentDetails}
	<DeploymentSteps {dotrain} deployment={deploymentName} {deploymentDetails} />
{/if}
