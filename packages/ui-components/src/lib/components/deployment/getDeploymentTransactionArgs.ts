import type { Config } from '@wagmi/core';
import { getAccount } from '@wagmi/core';
import type {
	DepositAndAddOrderCalldataResult,
	DotrainOrderGui
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order',
	MISSING_GUI = 'Order GUI is required',
	MISSING_CONFIG = 'Wagmi config is required',
	NO_ACCOUNT_CONNECTED = 'No wallet address found',
	ERROR_GETTING_ARGS = 'Error getting deployment transaction args',
	ERROR_GETTING_NETWORK_KEY = 'Error getting network key'
}

export interface HandleAddOrderResult {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
}

export async function getDeploymentTransactionArgs(
	gui: DotrainOrderGui,
	account: string | null
): Promise<HandleAddOrderResult> {
	let network: string;

	const networkKeyResult = gui.getNetworkKey();
	if (networkKeyResult.error) {
		throw new Error(AddOrderErrors.ERROR_GETTING_NETWORK_KEY);
	}
	network = networkKeyResult.value;
	
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
		orderbookAddress: orderbookAddress as Hex,
		chainId,
		network
	};
}
