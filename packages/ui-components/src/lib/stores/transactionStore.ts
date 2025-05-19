import { writable } from 'svelte/store';
import type { Hex } from 'viem';
import { sendTransaction, switchChain, waitForTransactionReceipt } from '@wagmi/core';
import { getExplorerLink } from '../services/getExplorerLink';
import { TransactionStatusMessage } from '$lib/types/transaction';
import type {
	DeploymentTransactionArgs,
	DepositOrWithdrawTransactionArgs,
	TransactionState
} from '$lib/types/transaction';
import {
	awaitSubgraphIndexing,
	getNewOrderConfig,
	getTransactionConfig
} from '$lib/services/awaitTransactionIndexing';

export const ADDRESS_ZERO = '0x0000000000000000000000000000000000000000';
export const ONE = BigInt('1000000000000000000');

export enum TransactionErrorMessage {
	BAD_CALLLDATA = 'Bad calldata.',
	DEPLOY_FAILED = 'Lock transaction failed.',
	TIMEOUT = 'The subgraph took too long to respond.',
	APPROVAL_FAILED = 'Approval transaction failed.',
	USER_REJECTED_APPROVAL = 'User rejected approval transaction.',
	USER_REJECTED_TRANSACTION = 'User rejected the transaction.',
	DEPLOYMENT_FAILED = 'Deployment transaction failed.',
	SWITCH_CHAIN_FAILED = 'Failed to switch chain.',
	DEPOSIT_FAILED = 'Failed to deposit tokens.',
	WITHDRAWAL_FAILED = 'Failed to withdraw tokens.',
	REMOVE_ORDER_FAILED = 'Failed to remove order.'
}

export type TransactionStore = {
	subscribe: (run: (value: TransactionState) => void) => () => void;
	reset: () => void;
	handleDeploymentTransaction: (args: DeploymentTransactionArgs) => Promise<void>;
	handleDepositOrWithdrawTransaction: (args: DepositOrWithdrawTransactionArgs) => Promise<void>;
	checkingWalletAllowance: (message?: string) => void;
	awaitWalletConfirmation: (message?: string) => void;
	awaitApprovalTx: (hash: string) => void;
	transactionSuccess: (hash: string, message?: string) => void;
	transactionError: (message: TransactionErrorMessage, hash?: string) => void;
};

export const initialState: TransactionState = {
	status: TransactionStatusMessage.IDLE,
	error: '',
	hash: '',
	data: null,
	functionName: '',
	message: '',
	newOrderHash: '',
	network: '',
	explorerLink: ''
};

const transactionStore = () => {
	const { subscribe, set, update } = writable(initialState);
	const reset = () => set(initialState);

	const awaitTransactionIndexing = async (
		subgraphUrl: string,
		txHash: string,
		successMessage: string
	) => {
		update((state) => ({
			...state,
			status: TransactionStatusMessage.PENDING_SUBGRAPH,
			message: 'Waiting for transaction to be indexed...'
		}));

		const result = await awaitSubgraphIndexing(
			getTransactionConfig(subgraphUrl, txHash, successMessage)
		);

		if (result.error) {
			return transactionError(TransactionErrorMessage.TIMEOUT);
		}

		if (result.value) {
			return transactionSuccess(result.value.txHash, result.value.successMessage);
		}
	};

	const awaitNewOrderIndexing = async (subgraphUrl: string, txHash: string, network?: string) => {
		update((state) => ({
			...state,
			status: TransactionStatusMessage.PENDING_SUBGRAPH,
			message: 'Waiting for new order to be indexed...'
		}));

		const result = await awaitSubgraphIndexing(getNewOrderConfig(subgraphUrl, txHash, '', network));

		if (result.error) {
			return transactionError(TransactionErrorMessage.TIMEOUT);
		}

		if (result.value) {
			return transactionSuccess(
				result.value.txHash,
				result.value.successMessage,
				result.value.orderHash,
				result.value.network
			);
		}
	};

	const checkingWalletAllowance = (message?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatusMessage.CHECKING_ALLOWANCE,
			message: message || ''
		}));
	const awaitWalletConfirmation = (message?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatusMessage.PENDING_WALLET,
			message: message || ''
		}));
	const awaitApprovalTx = (hash: string, symbol: string | undefined) =>
		update((state) => ({
			...state,
			hash: hash,
			status: TransactionStatusMessage.PENDING_APPROVAL,
			message: `Approving ${symbol || 'token'} spend...`
		}));
	const awaitTx = (
		hash: string,
		status: TransactionStatusMessage,
		explorerLink?: string,
		message?: string
	) =>
		update((state) => ({
			...state,
			hash: hash,
			status: status,
			message: message || 'Waiting for transaction...',
			explorerLink: explorerLink || ''
		}));
	const transactionSuccess = (
		hash: string,
		message?: string,
		newOrderHash?: string,
		network?: string
	) => {
		update((state) => ({
			...state,
			status: TransactionStatusMessage.SUCCESS,
			hash: hash,
			message: message || '',
			newOrderHash: newOrderHash || '',
			network: network || ''
		}));
	};
	const transactionError = (error: TransactionErrorMessage, hash?: string) =>
		update((state) => ({
			...state,
			status: TransactionStatusMessage.ERROR,
			error,
			message: error,
			hash: hash ?? ''
		}));

	const handleDeploymentTransaction = async ({
		config,
		approvals,
		deploymentCalldata,
		orderbookAddress,
		chainId,
		subgraphUrl,
		network
	}: DeploymentTransactionArgs) => {
		reset();
		try {
			await switchChain(config, { chainId });
		} catch {
			return transactionError(TransactionErrorMessage.SWITCH_CHAIN_FAILED);
		}
		for (const approval of approvals) {
			let approvalHash: Hex;
			try {
				awaitWalletConfirmation(
					`Please approve ${approval.symbol || approval.token} spend in your wallet...`
				);
				approvalHash = await sendTransaction(config, {
					to: approval.token as `0x${string}`,
					data: approval.calldata as `0x${string}`
				});
			} catch {
				return transactionError(TransactionErrorMessage.USER_REJECTED_APPROVAL);
			}
			try {
				awaitApprovalTx(approvalHash, approval.symbol);
				await waitForTransactionReceipt(config, { hash: approvalHash });
			} catch {
				return transactionError(TransactionErrorMessage.APPROVAL_FAILED);
			}
		}

		let hash: Hex;
		try {
			awaitWalletConfirmation('Please confirm deployment in your wallet...');
			hash = await sendTransaction(config, {
				to: orderbookAddress as `0x${string}`,
				data: deploymentCalldata as `0x${string}`
			});
		} catch {
			return transactionError(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
		}
		try {
			const transactionExplorerLink = await getExplorerLink(hash, chainId, 'tx');
			awaitTx(hash, TransactionStatusMessage.PENDING_DEPLOYMENT, transactionExplorerLink);
			await waitForTransactionReceipt(config, { hash });
			if (subgraphUrl) {
				return awaitNewOrderIndexing(subgraphUrl, hash, network);
			}
			return transactionSuccess(
				hash,
				'Deployment successful. Check the Orders page for your new order.',
				'',
				network
			);
		} catch {
			return transactionError(TransactionErrorMessage.DEPLOYMENT_FAILED);
		}
	};

	const handleDepositOrWithdrawTransaction = async ({
		config,
		approvalCalldata,
		transactionCalldata,
		action,
		chainId,
		vault,
		subgraphUrl
	}: DepositOrWithdrawTransactionArgs) => {
		reset();

		try {
			await switchChain(config, { chainId });
		} catch {
			return transactionError(TransactionErrorMessage.SWITCH_CHAIN_FAILED);
		}
		if (approvalCalldata) {
			let approvalHash: Hex;
			try {
				awaitWalletConfirmation(`Please approve ${vault.token.symbol} spend in your wallet...`);
				approvalHash = await sendTransaction(config, {
					to: vault.token.address as `0x${string}`,
					data: approvalCalldata as unknown as `0x${string}`
				});
			} catch {
				return transactionError(TransactionErrorMessage.USER_REJECTED_APPROVAL);
			}
			try {
				awaitApprovalTx(approvalHash, vault.token.symbol);
				await waitForTransactionReceipt(config, { hash: approvalHash });
			} catch {
				return transactionError(TransactionErrorMessage.APPROVAL_FAILED);
			}
		}
		let hash: Hex;
		try {
			awaitWalletConfirmation(
				`Please confirm ${action === 'deposit' ? 'deposit' : 'withdrawal'} in your wallet...`
			);
			hash = await sendTransaction(config, {
				to: vault.orderbook.id as `0x${string}`,
				data: transactionCalldata as unknown as `0x${string}`
			});
		} catch {
			return transactionError(TransactionErrorMessage.USER_REJECTED_TRANSACTION);
		}
		try {
			const transactionExplorerLink = await getExplorerLink(hash, chainId, 'tx');
			awaitTx(
				hash,
				action === 'deposit'
					? TransactionStatusMessage.PENDING_DEPOSIT
					: TransactionStatusMessage.PENDING_WITHDRAWAL,
				transactionExplorerLink
			);
			await waitForTransactionReceipt(config, { hash });
			return awaitTransactionIndexing(
				subgraphUrl,
				hash,
				`The ${action === 'deposit' ? 'deposit' : 'withdrawal'} was successful.`
			);
		} catch {
			return transactionError(
				action === 'deposit'
					? TransactionErrorMessage.DEPOSIT_FAILED
					: TransactionErrorMessage.WITHDRAWAL_FAILED
			);
		}
	};

	return {
		subscribe,
		reset,
		handleDeploymentTransaction,
		handleDepositOrWithdrawTransaction,
		checkingWalletAllowance,
		awaitWalletConfirmation,
		awaitApprovalTx,
		transactionSuccess,
		transactionError,
		awaitTransactionIndexing,
		awaitNewOrderIndexing
	};
};

export default transactionStore();
