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
	NO_WALLET = 'No wallet address found'
}

export interface HandleAddOrderResult {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
}

export async function getDeploymentTransactionArgs(
	gui: DotrainOrderGui | null,
	wagmiConfig: Config | undefined
): Promise<HandleAddOrderResult> {
	if (!gui) {
		throw new Error(AddOrderErrors.MISSING_GUI);
	}

	if (!wagmiConfig) {
		throw new Error(AddOrderErrors.MISSING_CONFIG);
	}

	const { address } = getAccount(wagmiConfig);
	if (!address) {
		throw new Error(AddOrderErrors.NO_WALLET);
	}

	const { approvals, deploymentCalldata, orderbookAddress, chainId } =
		await gui.getDeploymentTransactionArgs(address);

	return {
		approvals,
		deploymentCalldata,
		orderbookAddress,
		chainId
	};
}
