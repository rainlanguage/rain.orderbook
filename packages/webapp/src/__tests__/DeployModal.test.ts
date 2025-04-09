import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import DeployModal from '../lib/components/DeployModal.svelte';
import type { DeployModalProps } from '@rainlanguage/ui-components';
import type { Config } from 'wagmi';

const { mockTransactionStore, mockWagmiConfigStore } = await vi.hoisted(
	() => import('@rainlanguage/ui-components')
);

vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...(await importOriginal()),
		transactionStore: mockTransactionStore
	};
});

vi.mock('../lib/stores/wagmi', () => {
	return {
		wagmiConfig: mockWagmiConfigStore
	};
});

describe('DeployModal', () => {
	const mockProps = {
		open: true,
		args: {
			approvals: {
				approvalCalldata: '0x',
				token: '0x',
				spender: '0x'
			},
			deploymentCalldata: '0x',
			orderbookAddress: '0x123' as const,
			chainId: 1,
			subgraphUrl: 'https://example.com',
			network: 'mainnet'
		}
	} as unknown as DeployModalProps;

	it('renders and initiates transaction handling', () => {
		const mockConfig = {
			appName: 'TESTCONFIG'
		};

		mockWagmiConfigStore.mockSetSubscribeValue(mockConfig as unknown as Config);
		const handleDeploymentTransactionSpy = vi.spyOn(
			mockTransactionStore,
			'handleDeploymentTransaction'
		);

		render(DeployModal, { props: mockProps });

		expect(handleDeploymentTransactionSpy).toHaveBeenCalledWith({
			config: mockConfig,
			...mockProps.args
		});
	});
});
