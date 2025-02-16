import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';
import type { DepositAndAddOrderCalldataResult } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';

export type DeploymentArgs = {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
	network: string;
};
