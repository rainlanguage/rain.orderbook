import type {
	DepositAndAddOrderCalldataResult,
	DotrainOrderGui
} from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order'
}

export interface HandleAddOrderResult {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
}

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
