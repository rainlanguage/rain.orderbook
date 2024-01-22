export interface ToastPayload {
  message_type: 'Success' | 'Error' | 'Warning' | 'Info';
  text: string;
}

export type ToastData = ToastPayload & { timestamp: Date; id: string };

export type ToastDataStore = { [id: string]: ToastData };