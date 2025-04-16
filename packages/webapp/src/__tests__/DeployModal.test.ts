import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/svelte';
import DeployModal from '../lib/components/DeployModal.svelte';
import type { DeployModalProps } from '@rainlanguage/ui-components';
import { mockWeb3Config } from '$lib/__mocks__/mockWeb3Config';

const { mockTransactionStore } = await vi.hoisted(() => import('@rainlanguage/ui-components'));
const { mockWagmiConfigStore } = await vi.hoisted(() => import('../lib/__mocks__/stores'));
vi.mock('@rainlanguage/ui-components', async (importOriginal) => {
	return {
		...(await importOriginal()),
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
		const handleDeploymentTransactionSpy = vi.spyOn(
			mockTransactionStore,
			'handleDeploymentTransaction'
		);

		render(DeployModal, { props: mockProps });

		expect(handleDeploymentTransactionSpy).toHaveBeenCalledWith(
			expect.objectContaining({
				config: mockWeb3Config,
				...mockProps.args
			})
		);
	});
});
