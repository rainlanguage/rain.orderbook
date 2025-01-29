import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import DeployModal from '../lib/components/DeployModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';

export type DeployModalProps = ComponentProps<DeployModal>;
const { mockTransactionStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const { mockWeb3Config } = await vi.hoisted(() => import('../lib/__mocks__/mockWeb3Config'));
vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...await importOriginal(),
		transactionStore: mockTransactionStore
	};
});

vi.mock('$lib/stores/wagmi', async (importOriginal) => {
	return {
		...await importOriginal(),
		wagmiConfig: mockWeb3Config
	};
});

describe('DeployModal', () => {
	const mockProps = {
		open: true,
		approvals: {
			approvalCalldata: '0x',
			token: '0x',
			spender: '0x'
		},
		deploymentCalldata: {
			calldata: '0x',
			value: 0n
		},
		orderbookAddress: '0x123' as const,
		chainId: 1
	} as unknown as DeployModalProps;

	beforeEach(() => {
		vi.clearAllMocks();
	});

	it('renders and initiates transaction handling', () => {
		const handleDeploymentTransactionSpy = vi.spyOn(transactionStore, 'handleDeploymentTransaction')
		render(DeployModal, { props: mockProps });
		expect(handleDeploymentTransactionSpy).toHaveBeenCalledWith({
			config: mockWeb3Config,
			approvals: mockProps.approvals,
			deploymentCalldata: mockProps.deploymentCalldata,
			orderbookAddress: mockProps.orderbookAddress,
			chainId: mockProps.chainId
		});
	});
});
