import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render } from '@testing-library/svelte';
import DeployModal from '../lib/components/DeployModal.svelte';
import { transactionStore } from '@rainlanguage/ui-components';
import type { ComponentProps } from 'svelte';
import { get } from 'svelte/store';

export type DeployModalProps = ComponentProps<DeployModal>;
const { mockTransactionStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const { mockWagmiConfigStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));
vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...await importOriginal(),
		transactionStore: mockTransactionStore
	};
});

vi.mock('$lib/stores/wagmi', () => {
	return {
		wagmiConfig: mockWagmiConfigStore
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
		const config = get(mockWagmiConfigStore);
		const handleDeploymentTransactionSpy = vi.spyOn(transactionStore, 'handleDeploymentTransaction')
		render(DeployModal, { props: mockProps });
		expect(handleDeploymentTransactionSpy).toHaveBeenCalledWith({
			config: config,
			approvals: mockProps.approvals,
			deploymentCalldata: mockProps.deploymentCalldata,
			orderbookAddress: mockProps.orderbookAddress,
			chainId: mockProps.chainId
		});
	});
});
