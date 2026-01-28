/* this file is auto-generated, do not modify */

export enum ToastMessageType {
  Success = 0,
  Error = 1,
  Warning = 2,
  Info = 3,
}
export interface ToastPayload {
  message_type: ToastMessageType;
  text: string;
}

export type TransactionStatus =
  | { type: 'Initialized' }
  | { type: 'PendingPrepare' }
  | { type: 'PendingSign' }
  | { type: 'Sending' }
  | { type: 'Confirmed'; payload: string }
  | { type: 'Failed'; payload: string };

export interface TransactionStatusNotice {
  id: string;
  status: TransactionStatus;
  chain_id: number;
  created_at: string;
  label: string;
}
