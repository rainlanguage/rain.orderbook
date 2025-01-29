import { writable } from 'svelte/store';
import { TransactionStatus, TransactionErrorMessage } from '../stores/transactionStore';

type MockTransactionStoreState = {
	status: TransactionStatus;
	error: string;
	hash: string;
	data: null;
	functionName: string;
	message: string;
};

const initialState: MockTransactionStoreState = {
	status: TransactionStatus.IDLE,
	error: '',
	hash: '',
	data: null,
	functionName: '',
	message: ''
};

const mockTransactionWritable = writable<MockTransactionStoreState>(initialState);

export const mockTransactionStore = {
	subscribe: mockTransactionWritable.subscribe,
	set: mockTransactionWritable.set,
	reset: () => mockTransactionWritable.set(initialState),

	handleDeploymentTransaction: async () => {
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			message: 'Strategy deployed successfully!',
			hash: '0x123'
		}))
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
			message: ''
		})),

	transactionSuccess: (hash: string, message: string = '') =>
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.SUCCESS,
			hash,
			message
		})),

	transactionError: (error: TransactionErrorMessage, hash: string = '') =>
		mockTransactionWritable.update((state) => ({
			...state,
			status: TransactionStatus.ERROR,
			error,
			hash
		})),

	mockSetSubscribeValue: (value: Partial<MockTransactionStoreState>) =>
		mockTransactionWritable.update((state) => ({
			...state,
			...value
		}))
};
