import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import DeployModal from '../lib/components/DeployModal.svelte';
import { get } from 'svelte/store';
import type { DeployModalProps } from '@rainlanguage/ui-components';

const { mockTransactionStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const { mockWagmiConfigStore } = await vi.hoisted(() => import('$lib/__mocks__/stores'));
vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...(await importOriginal()),
		useSignerAddress: vi.fn().mockReturnValue({
			signerAddress: '0x123'
		}),
		wagmiConfig: mockWagmiConfigStore,
		transactionStore: mockTransactionStore
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
			deploymentCalldata: {
				calldata: '0x',
				value: 0n
			},
			orderbookAddress: '0x123' as const,
			chainId: 1,
			subgraphUrl: 'https://example.com',
			network: 'mainnet'
		}
	} as unknown as DeployModalProps;

	it('renders and initiates transaction handling', () => {
		const config = get(mockWagmiConfigStore);
		const handleDeploymentTransactionSpy = vi.spyOn(
			mockTransactionStore,
			'handleDeploymentTransaction'
		);

		render(DeployModal, { props: mockProps });

		expect(handleDeploymentTransactionSpy).toHaveBeenCalledWith({
			config,
			...mockProps.args
		});
	});
});
