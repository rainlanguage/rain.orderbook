import { registryUrl } from '$lib/stores/registry';
import { rawDotrain } from '$lib/stores/raw-dotrain';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { get } from 'svelte/store';
import type { PageLoad } from './$types';
import { redirect } from '@sveltejs/kit';

export const load: PageLoad = async ({ fetch, params, parent }) => {
	const { strategyName, deploymentKey } = params;
	const { registry } = await parent();
	if (registry) {
		registryUrl.set(registry);
	}
	try {
		let dotrain;
		if (strategyName === 'raw' && get(rawDotrain)) {
			dotrain = get(rawDotrain);
		} else {
			const _registryUrl = get(registryUrl);
			const response = await fetch(_registryUrl);
			const files = await response.text();

			const fileList = files
				.split('\n')
				.filter(Boolean)
				.map((line: string) => {
					const [name, url] = line.split(' ');
					return { name, url };
				});

			const strategy = fileList.find((file: { name: string }) => file.name === strategyName);
			if (!strategy) {
				throw new Error(`Strategy ${strategyName} not found`);
			}

			const dotrainResponse = await fetch(strategy.url);
			dotrain = await dotrainResponse.text();
		}

		// Process deployments for both raw and registry strategies
		const deploymentWithDetails = await DotrainOrderGui.getDeploymentDetails(dotrain);
		const deployments = Array.from(deploymentWithDetails, ([key, details]) => ({
			key,
			...details
		}));

		const deployment = deployments.find(
			(deployment: { key: string }) => deployment.key === deploymentKey
		);

		if (!deployment) {
			throw new Error(`Deployment ${deploymentKey} not found`);
		}

		const { key, name, description } = deployment;

		return {
			dotrain,
			strategyName,
			key,
			name,
			description
		};
	} catch {
		throw redirect(307, '/deploy');
	}
};
