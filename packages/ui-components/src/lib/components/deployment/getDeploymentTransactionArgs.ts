import type {
	DepositAndAddOrderCalldataResult,
	DotrainOrderGui
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order',
	MISSING_GUI = 'Order GUI is required',
}

export interface HandleAddOrderResult {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
}

/**
 * Gets the transaction arguments needed for deploying an order
 * @param gui The order GUI instance containing the order configuration
 * @param account The wallet address of the user deploying the order
 * @returns Transaction arguments including approvals, deployment calldata, orderbook address and chain ID
 * @throws {Error} If getting deployment transaction args fails or if there are validation errors
 */

export async function getDeploymentTransactionArgs(
	gui: DotrainOrderGui,
	account: Hex
): Promise<HandleAddOrderResult> {
	const result = await gui.getDeploymentTransactionArgs(account);
	if (result.error) {
		throw new Error(result.error.msg);
	}
	const { approvals, deploymentCalldata, orderbookAddress, chainId } = result.value;

	return {
		approvals,
		deploymentCalldata,
		orderbookAddress: orderbookAddress as Hex,
		chainId
	};
}
