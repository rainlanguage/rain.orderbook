import { registryUrl } from '$lib/stores/registry';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { get } from 'svelte/store';

export const load = async ({
	fetch,
	params
}: {
	fetch: typeof globalThis.fetch;
	params: { strategyName: string; deploymentKey: string };
}) => {
	try {
		const _registryUrl = get(registryUrl);
		const response = await fetch(_registryUrl);
		console.log(_registryUrl);
		const files = await response.text();
		const { strategyName, deploymentKey } = params;

		const fileList = files
			.split('\n')
			.filter(Boolean)
			.map((line: string) => {
				const [name, url] = line.split(' ');
				return { name, url };
			});

		const strategy = fileList.find((file: { name: string }) => file.name === strategyName);

		if (strategy) {
			const dotrainResponse = await fetch(strategy.url);
			const dotrain = await dotrainResponse.text();

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
		} else {
			return {
				dotrain: null,
				strategyName: null,
				deploymentKey: null
			};
		}
	} catch {
		return {
			dotrain: null,
			strategyName: null,
			deploymentKey: null
		};
	}
};
