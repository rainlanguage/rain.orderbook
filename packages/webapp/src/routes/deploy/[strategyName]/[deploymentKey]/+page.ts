import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, params }) => {
	try {
		const response = await fetch(
			'https://raw.githubusercontent.com/rainlanguage/rain.strategies/refs/heads/main/strategies/dev/registry'
		);
		const files = await response.text();
		const { strategyName, deploymentKey } = params;

		const fileList = files
			.split('\n')
			.filter(Boolean)
			.map((line) => {
				const [name, url] = line.split(' ');
				return { name, url };
			});

		const strategy = fileList.find((file) => file.name === strategyName);

		if (strategy) {
			const dotrainResponse = await fetch(strategy.url);
			const dotrain = await dotrainResponse.text();

			const deploymentWithDetails = await DotrainOrderGui.getDeploymentDetails(dotrain);

			const deployments = Array.from(deploymentWithDetails, ([key, details]) => ({
				key,
				...details
			}));
			const deployment = deployments.find((deployment) => deployment.key === deploymentKey);

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
