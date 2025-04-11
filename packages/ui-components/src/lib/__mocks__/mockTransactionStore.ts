import { writable } from 'svelte/store';
import {
	TransactionStatus,
	initialState,
	TransactionErrorMessage,
	type TransactionState,
	type TransactionStore
} from '../stores/transactionStore';

const mockTransactionWritable = writable<TransactionState>(initialState);

export const mockTransactionStore: Partial<TransactionStore> & {
	mockSetSubscribeValue: (value: Partial<TransactionState>) => void;
} = {
	subscribe: mockTransactionWritable.subscribe,
	reset: () => mockTransactionWritable.set(initialState),

	handleDeploymentTransaction: async () => {
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			message: 'Strategy deployed successfully!',
			hash: '0x123'
		}));
		return Promise.resolve();
	},

	handleDepositOrWithdrawTransaction: async () => {
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			message: 'Transaction successful!',
			hash: '0x456'
		}));
		return Promise.resolve();
	},

	handleRemoveOrderTransaction: async () => {
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			message: 'Order removed successfully',
			hash: '0x789'
		}));
		return Promise.resolve();
	},

	checkingWalletAllowance: (message: string = '') =>
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.CHECKING_ALLOWANCE,
			message
		})),

	awaitWalletConfirmation: (message: string = '') =>
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.PENDING_WALLET,
			message
		})),

	awaitApprovalTx: (hash: string) =>
		mockTransactionWritable.update((state) => ({
			...state,
			hash,
			status: TransactionStatus.PENDING_APPROVAL,
			message: 'Approving token spend...'
		})),

	transactionSuccess: (
		hash: string,
		message: string = '',
		newOrderHash: string = '',
		network: string = ''
	) =>
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			hash,
			message,
			newOrderHash,
			network
		})),

	transactionError: (error: TransactionErrorMessage, hash: string = '') =>
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.ERROR,
			error,
			hash
		})),

	// Extra function for testing purposes
	mockSetSubscribeValue: (value: Partial<TransactionState>) =>
		mockTransactionWritable.update((state) => ({
			...state,
			...value
		}))
};
