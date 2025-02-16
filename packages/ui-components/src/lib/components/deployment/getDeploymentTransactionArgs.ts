import type { Config } from '@wagmi/core';
import { getAccount } from '@wagmi/core';
import type {
	DepositAndAddOrderCalldataResult,
	DotrainOrderGui
} from '@rainlanguage/orderbook/js_api';
import type { OrderIO } from '@rainlanguage/orderbook/js_api';
import type { Hex } from 'viem';
import type { ExtendedApprovalCalldata } from '$lib/stores/transactionStore';

export enum AddOrderErrors {
	ADD_ORDER_FAILED = 'Failed to add order',
	MISSING_GUI = 'Order GUI is required',
	MISSING_CONFIG = 'Wagmi config is required',
	NO_WALLET = 'No wallet address found',
	INVALID_CHAIN_ID = 'Invalid chain ID in deployment',
	MISSING_ORDERBOOK = 'Orderbook address not found',
	TOKEN_INFO_FAILED = 'Failed to fetch token information',
	APPROVAL_FAILED = 'Failed to generate approval calldata',
	DEPLOYMENT_FAILED = 'Failed to generate deployment calldata'
}

export interface HandleAddOrderResult {
	approvals: ExtendedApprovalCalldata[];
	deploymentCalldata: DepositAndAddOrderCalldataResult;
	orderbookAddress: Hex;
	chainId: number;
}

export async function getDeploymentTransactionArgs(
	gui: DotrainOrderGui | null,
	wagmiConfig: Config | undefined,
	allTokenOutputs: OrderIO[]
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

	let approvalResults;
	try {
		approvalResults = await gui.generateApprovalCalldatas(address);
	} catch (error) {
		throw new Error(
			`${AddOrderErrors.APPROVAL_FAILED}: ${error instanceof Error ? error.message : 'Unknown error'}`
		);
	}

	let deploymentCalldata;
	try {
		deploymentCalldata = await gui.generateDepositAndAddOrderCalldatas();
	} catch (error) {
		throw new Error(
			`${AddOrderErrors.DEPLOYMENT_FAILED}: ${error instanceof Error ? error.message : 'Unknown error'}`
		);
	}

	const currentDeployment = gui.getCurrentDeployment();
	const chainId = currentDeployment?.deployment?.order?.network?.['chain-id'] as number;
	if (!chainId) {
		throw new Error(AddOrderErrors.INVALID_CHAIN_ID);
	}

	// @ts-expect-error orderbook is not typed
	const orderbookAddress = currentDeployment?.deployment?.order?.orderbook?.address;
	if (!orderbookAddress) {
		throw new Error(AddOrderErrors.MISSING_ORDERBOOK);
	}

	let approvals: ExtendedApprovalCalldata[] = [];

	try {
		const outputTokenInfos = await Promise.all(
			allTokenOutputs.map((token) => {
				const key = token.token?.key;
				if (!key) throw new Error('Token key is missing');
				return gui.getTokenInfo(key);
			})
		);

		if (approvalResults !== 'NoDeposits') {
			approvals = approvalResults.Calldatas.map((approval) => {
				const token = outputTokenInfos.find((token) => token?.address === approval.token);
				if (!token) {
					throw new Error(`Token info not found for address: ${approval.token}`);
				}
				return {
					...approval,
					symbol: token.symbol
				};
			});
		}
	} catch (error) {
		throw new Error(
			`${AddOrderErrors.TOKEN_INFO_FAILED}: ${error instanceof Error ? error.message : 'Unknown error'}`
		);
	}

	return {
		approvals,
		deploymentCalldata,
		orderbookAddress,
		chainId
	};
}
