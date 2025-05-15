import { wagmiConfig } from '$lib/stores/wagmi';
import { sendTransaction, switchChain } from '@wagmi/core';
import type { Hex } from 'viem';
import type { TransactionConfirmationProps } from '@rainlanguage/ui-components';
import { get } from 'svelte/store';

export type WalletConfirmationState =
	| { status: 'awaiting_confirmation' }
	| { status: 'confirmed' }
	| { status: 'rejected'; reason: string }
	| { status: 'error'; reason: string };

export async function handleWalletConfirmation(
	args: TransactionConfirmationProps['args']
): Promise<{ state: WalletConfirmationState; hash?: Hex }> {
	try {
		await switchChain(get(wagmiConfig), { chainId: args.chainId });
	} catch (error) {
		return {
			state: {
				status: 'error',
				reason: error instanceof Error ? error.message : 'Failed to switch chain'
			}
		};
	}

	try {
		const calldata = await args.getCalldataFn();

		const transactionHash = await sendTransaction(get(wagmiConfig), {
			to: args.orderbookAddress,
			data: calldata as Hex
		});

		args.onConfirm(transactionHash);

		return {
			state: { status: 'confirmed' },
			hash: transactionHash
		};
	} catch {
		return {
			state: {
				status: 'rejected',
				reason: 'User rejected transaction'
			}
		};
	}
}
