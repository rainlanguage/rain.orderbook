import type { DepositAndAddOrderCalldataResult } from '@rainlanguage/orderbook/js_api';
import type { ApprovalCalldataResult } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';

export type DeploymentArgs = {
	approvals: ApprovalCalldataResult;
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
	subgraphUrl: string;
};
