import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest';
import { handleDeployment } from '../lib/utils/handleDeployment';
import {
	getDeploymentTransactionArgs,
	type HandleAddOrderResult
} from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import type { DeploymentHandlers } from '../lib/utils/handleDeployment';

// Mock the getDeploymentTransactionArgs function
vi.mock('../lib/components/deployment/getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

describe('handleDeployment', () => {
	// Mock data
	const mockGui = {} as DotrainOrderGui;
	const mockAccount = '0x1234567890123456789012345678901234567890';
	const mockSubgraphUrl = 'https://example.com/subgraph';

	// Mock handlers
	const mockHandlers: DeploymentHandlers = {
		handleDisclaimerModal: vi.fn(),
		handleDeployModal: vi.fn()
	};

	// Mock deployment transaction args
	const mockDeploymentArgs = {
		to: '0xcontract',
		data: '0xdata',
		value: BigInt(0)
	} as unknown as HandleAddOrderResult;

	beforeEach(() => {
		// Reset all mocks before each test
		vi.resetAllMocks();

		// Setup default mock implementation for getDeploymentTransactionArgs
		vi.mocked(getDeploymentTransactionArgs).mockResolvedValue(mockDeploymentArgs);
	});

	it('should call getDeploymentTransactionArgs with correct parameters', async () => {
		await handleDeployment(mockGui, mockAccount, mockHandlers, mockSubgraphUrl);

		expect(getDeploymentTransactionArgs).toHaveBeenCalledWith(mockGui, mockAccount);
	});

	it('should show disclaimer modal when handleDisclaimerModal is provided', async () => {
		await handleDeployment(mockGui, mockAccount, mockHandlers, mockSubgraphUrl);

		expect(mockHandlers.handleDisclaimerModal).toHaveBeenCalledWith({
			open: true,
			onAccept: expect.any(Function)
		});
	});

	it('should not show disclaimer modal when handleDisclaimerModal is not provided', async () => {
		const handlersWithoutDisclaimer: DeploymentHandlers = {
			handleDeployModal: vi.fn()
		};

		await handleDeployment(mockGui, mockAccount, handlersWithoutDisclaimer, mockSubgraphUrl);

		expect(handlersWithoutDisclaimer.handleDeployModal).toHaveBeenCalledWith({
			open: true,
			args: {
				...mockDeploymentArgs,
				subgraphUrl: mockSubgraphUrl
			}
		});
	});

	it('should call handleDeployModal with correct args when disclaimer is accepted', async () => {
		await handleDeployment(mockGui, mockAccount, mockHandlers, mockSubgraphUrl);

		// Get the onAccept callback from the handleDisclaimerModal call
		const onAcceptCallback = (vi.mocked(mockHandlers.handleDisclaimerModal) as Mock).mock
			.calls[0][0].onAccept;

		// Call the onAccept callback
		onAcceptCallback();

		// Verify handleDeployModal was called with the correct args
		expect(mockHandlers.handleDeployModal).toHaveBeenCalledWith({
			open: true,
			args: {
				...mockDeploymentArgs,
				subgraphUrl: mockSubgraphUrl
			}
		});
	});

	it('should propagate errors from getDeploymentTransactionArgs', async () => {
		const mockError = new Error('Deployment failed');
		vi.mocked(getDeploymentTransactionArgs).mockRejectedValue(mockError);

		await expect(
			handleDeployment(mockGui, mockAccount, mockHandlers, mockSubgraphUrl)
		).rejects.toThrow(mockError);
	});

	it('should not proceed with deployment when disclaimer is denied', async () => {
		let disclaimerShown = false;
		let deployModalCalled = false;

		const handlersWithDenialTracking: DeploymentHandlers = {
			handleDisclaimerModal: vi.fn().mockImplementation(({ open, onDeny }) => {
				disclaimerShown = open;

				if (onDeny) {
					onDeny();
				}
			}),
			handleDeployModal: vi.fn().mockImplementation(() => {
				deployModalCalled = true;
			})
		};

		await handleDeployment(mockGui, mockAccount, handlersWithDenialTracking, mockSubgraphUrl);

		expect(disclaimerShown).toBe(true);

		expect(deployModalCalled).toBe(false);
		expect(handlersWithDenialTracking.handleDeployModal).not.toHaveBeenCalled();
	});
});
