// Define the Toast interface
export interface ToastProps {
	message: string;
	type: 'success' | 'error' | 'warning' | 'info';
	color: 'green' | 'red' | 'yellow' | 'blue';
}
