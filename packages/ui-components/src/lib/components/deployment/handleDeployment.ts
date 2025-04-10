import type { DotrainOrderGui } from '@rainlanguage/orderbook';
import type { Hex } from 'viem';
import type { DeploymentArgs } from '$lib/types/transaction';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order',
	MISSING_GUI = 'Order GUI is required',
	MISSING_CONFIG = 'Wagmi config is required',
	NO_ACCOUNT_CONNECTED = 'No wallet address found',
	ERROR_GETTING_ARGS = 'Error getting deployment transaction args',
	ERROR_GETTING_NETWORK_KEY = 'Error getting network key'
}

export async function handleDeployment(
	gui: DotrainOrderGui,
	account: string | null,
	networkKey: string,
	subgraphUrl?: string
): Promise<DeploymentArgs> {
	if (!account) {
		throw new Error(AddOrderErrors.NO_ACCOUNT_CONNECTED);
	}

	const result = await gui.getDeploymentTransactionArgs(account);

	if (result.error) {
		throw new Error(result.error.msg);
	}

	const { approvals, deploymentCalldata, orderbookAddress, chainId } = result.value;

	return {
		approvals,
		deploymentCalldata,
		// Cast to Hex, since js_api returns a string
		orderbookAddress: orderbookAddress as Hex,
		chainId,
		network: networkKey,
		subgraphUrl
	};
}
