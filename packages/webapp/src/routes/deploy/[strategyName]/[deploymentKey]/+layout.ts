import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { LayoutLoad } from '../$types';

export const load: LayoutLoad = async ({ params, parent }) => {
	const { deploymentKey } = params;
	const { dotrain } = await parent();

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

	return { deployment, dotrain };
};
