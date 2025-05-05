// useToasts.ts
import { getToastsContext } from './context';

export function useToasts() {
  const toastsStore = getToastsContext();
  
  const addToast = (text: string) => {
    toastsStore.update(toasts => [...toasts, text]);
  };
  
  return {
    toasts: toastsStore,
    addToast
  };
}