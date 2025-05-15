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
	const config = get(wagmiConfig);
	let calldata: string;
	let transactionHash: Hex;
	try {
		await switchChain(config, { chainId: args.chainId });
	} catch (error) {
		return {
			state: {
				status: 'error',
				reason: error instanceof Error ? error.message : 'Failed to switch chain'
			}
		};
	}

	try {
		calldata = await args.getCalldataFn();
	} catch (error) {
		return {
			state: {
				status: 'error',
				reason: error instanceof Error ? error.message : 'Failed to get calldata'
			}
		};
	}

	try {
		transactionHash = await sendTransaction(config, {
			to: args.orderbookAddress,
			data: calldata as Hex
		});
	} catch (error) {
		return {
			state: {
				status: 'rejected',
				reason: error instanceof Error ? error.message : 'User rejected transaction'
			}
		};
	}

	args.onConfirm(transactionHash);
	return {
		state: { status: 'confirmed' },
		hash: transactionHash
	};
}
