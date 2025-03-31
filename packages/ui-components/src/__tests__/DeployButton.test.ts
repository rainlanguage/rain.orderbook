import { render, screen, fireEvent, waitFor } from '@testing-library/svelte';
import { vi, describe, it, expect, beforeEach, type Mock } from 'vitest';
import DeployButton from '../lib/components/deployment/DeployButton.svelte';
import { DeploymentStepsError, DeploymentStepsErrorCode } from '../lib/errors';
import * as getDeploymentTransactionArgsModule from '../lib/components/deployment/getDeploymentTransactionArgs';
import { DotrainOrderGui } from '@rainlanguage/orderbook/js_api';
import { type HandleAddOrderResult } from '../lib/components/deployment/getDeploymentTransactionArgs';
import type { ComponentProps } from 'svelte';
import { useGui } from '../lib/hooks/useGui';
import { useAccount } from '../lib/providers/wallet/useAccount';
import { writable } from 'svelte/store';

type DeployButtonProps = ComponentProps<DeployButton>;

vi.mock('../lib/providers/wallet/useAccount', () => ({
	useAccount: vi.fn()
}));

const mockHandleAddOrderResult: HandleAddOrderResult = {
	approvals: [],
	deploymentCalldata: '0x123',
	orderbookAddress: '0x456',
	chainId: 1337,
	network: 'testnet'
};

const mockGui = {
	generateDotrainText: vi.fn().mockReturnValue('mock dotrain text'),
	getCurrentDeployment: vi.fn().mockReturnValue({
		deployment: {
			order: {
				orderbook: {
					address: '0x456'
				}
			}
		}
	})
} as unknown as DotrainOrderGui;

const defaultProps: DeployButtonProps = {
	testId: 'deploy-button'
};

vi.mock('../lib/components/deployment/getDeploymentTransactionArgs', () => ({
	getDeploymentTransactionArgs: vi.fn()
}));

vi.mock('../lib/hooks/useGui', () => ({
	useGui: vi.fn()
}));

describe('DeployButton', () => {
	beforeEach(() => {
		vi.clearAllMocks();
		DeploymentStepsError.clear();
		(useGui as Mock).mockReturnValue(mockGui);
		(useAccount as Mock).mockReturnValue({
			account: writable('0x123')
		});
	});

	it('renders the deploy button correctly', () => {
		render(DeployButton, {
			props: defaultProps
		});

		expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		expect(screen.getByTestId('deploy-button')).toBeInTheDocument();
	});

	it('shows loading state when checking deployment', async () => {
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockImplementation(
			() => new Promise((resolve) => setTimeout(() => resolve(mockHandleAddOrderResult), 100))
		);

		render(DeployButton, {
			props: defaultProps
		});

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(screen.getByText('Checking deployment...')).toBeInTheDocument();
		});
	});

	it('calls getDeploymentTransactionArgs with correct arguments', async () => {
		render(DeployButton, { props: defaultProps });

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).toHaveBeenCalledWith(
				mockGui,
				'0x123'
			);
		});
	});

	it('applies custom testId when provided', () => {
		render(DeployButton, {
			props: {
				...defaultProps,
				testId: 'custom-test-id'
			}
		});

		expect(screen.getByTestId('custom-test-id')).toBeInTheDocument();
	});

	it('handles failed deployment transaction args correctly', async () => {
		const mockError = new Error('error getting args');
		vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockRejectedValue(
			mockError
		);

		const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');

		render(DeployButton, { props: defaultProps });

		fireEvent.click(screen.getByText('Deploy Strategy'));

		await waitFor(() => {
			expect(catchSpy).toHaveBeenCalledWith(mockError, DeploymentStepsErrorCode.ADD_ORDER_FAILED);
			expect(screen.getByText('Deploy Strategy')).toBeInTheDocument();
		});
	});

it('handles missing account correctly', async () => {
  (useAccount as Mock).mockReturnValue({
    account: writable(null)
  });
  
  const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');
  render(DeployButton, { props: defaultProps });
  
  fireEvent.click(screen.getByText('Deploy Strategy'));
  
  await waitFor(() => {
    // Expect some sort of error handling for missing account
    expect(catchSpy).toHaveBeenCalledWith(
      expect.any(Error),
      DeploymentStepsErrorCode.ADD_ORDER_FAILED
    );
  });
});

it('handles null GUI correctly', async () => {
  // Mock useGui to return null
  (useGui as Mock).mockReturnValue(null);
  
  const catchSpy = vi.spyOn(DeploymentStepsError, 'catch');
  render(DeployButton, { props: defaultProps });
  
  fireEvent.click(screen.getByText('Deploy Strategy'));
  
  await waitFor(() => {
    // Expect error handling for missing GUI
    expect(catchSpy).toHaveBeenCalledWith(
      expect.any(Error),
      DeploymentStepsErrorCode.ADD_ORDER_FAILED
    );
  });
});

it('clears deployment error when button is clicked', async () => {
  const clearSpy = vi.spyOn(DeploymentStepsError, 'clear');
  render(DeployButton, { props: defaultProps });
  
  fireEvent.click(screen.getByText('Deploy Strategy'));
  
  expect(clearSpy).toHaveBeenCalled();
});

it('disables button during checking deployment', async () => {
  vi.mocked(getDeploymentTransactionArgsModule.getDeploymentTransactionArgs).mockImplementation(
    () => new Promise((resolve) => setTimeout(() => resolve(mockHandleAddOrderResult), 100))
  );
  
  render(DeployButton, { props: defaultProps });
  
  const button = screen.getByTestId('deploy-button');
  fireEvent.click(button);
  
  await waitFor(() => {
    expect(button).toBeDisabled();
  });
  
  await waitFor(() => {
    expect(button).not.toBeDisabled();
  }, { timeout: 200 });
});
});
