import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { AddOrderErrors, handleDeployment } from '../lib/components/deployment/handleDeployment';
import type { DotrainOrderGui } from '@rainlanguage/orderbook';

describe('handleDeployment', () => {
	const mockAccount = '0x1234567890123456789012345678901234567890';
	const mockSubgraphUrl = 'https://example.com/subgraph';

	const mockGui: DotrainOrderGui = {
		getNetworkKey: vi.fn(),
		getDeploymentTransactionArgs: vi.fn()
	} as unknown as DotrainOrderGui;

	const mockApprovals = [{ tokenAddress: '0xtoken', amount: BigInt(1000) }];
	const mockDeploymentCalldata = {
		to: '0xcontract',
		data: '0xdata',
		value: BigInt(0)
	};
	const mockOrderbookAddress = '0xorderbook';
	const mockChainId = 1;
	const mockNetwork = 'mainnet';

	beforeEach(() => {
		vi.resetAllMocks();

		(mockGui.getNetworkKey as Mock).mockReturnValue({
			error: null,
			value: mockNetwork
		});

		(mockGui.getDeploymentTransactionArgs as Mock).mockResolvedValue({
			error: null,
			value: {
				approvals: mockApprovals,
				deploymentCalldata: mockDeploymentCalldata,
				orderbookAddress: mockOrderbookAddress,
				chainId: mockChainId
			}
		});
	});

	it('should return deployment data with correct parameters', async () => {
		const result = await handleDeployment(mockGui, mockAccount, mockSubgraphUrl);

		expect(result).toEqual({
			approvals: mockApprovals,
			deploymentCalldata: mockDeploymentCalldata,
			orderbookAddress: mockOrderbookAddress,
			chainId: mockChainId,
			network: mockNetwork,
			subgraphUrl: mockSubgraphUrl
		});

		expect(mockGui.getNetworkKey).toHaveBeenCalled();
		expect(mockGui.getDeploymentTransactionArgs).toHaveBeenCalledWith(mockAccount);
	});

	it('should throw an error if network key retrieval fails', async () => {
		(mockGui.getNetworkKey as Mock).mockReturnValue({
			error: new Error('Network key error'),
			value: null
		});

		await expect(handleDeployment(mockGui, mockAccount, mockSubgraphUrl)).rejects.toThrow(
			AddOrderErrors.ERROR_GETTING_NETWORK_KEY
		);
	});

	it('should throw an error if account is not provided', async () => {
		await expect(handleDeployment(mockGui, null, mockSubgraphUrl)).rejects.toThrow(
			AddOrderErrors.NO_ACCOUNT_CONNECTED
		);
	});

	it('should throw an error if deployment transaction args retrieval fails', async () => {
		(mockGui.getDeploymentTransactionArgs as Mock).mockResolvedValue({
			error: { msg: 'Deployment args error' },
			value: null
		});

		await expect(handleDeployment(mockGui, mockAccount, mockSubgraphUrl)).rejects.toThrow(
			'Deployment args error'
		);
	});

	it('should work without subgraphUrl', async () => {
		const result = await handleDeployment(mockGui, mockAccount);

		expect(result).toEqual({
			approvals: mockApprovals,
			deploymentCalldata: mockDeploymentCalldata,
			orderbookAddress: mockOrderbookAddress,
			chainId: mockChainId,
			network: mockNetwork,
			subgraphUrl: undefined
		});
	});
});
