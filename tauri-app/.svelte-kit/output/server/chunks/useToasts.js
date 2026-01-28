import { s as setContext, g as getContext } from "./ssr.js";
const TOASTS_KEY = "rain:ui-components:toasts";
function getToastsContext() {
  const toasts = getContext(TOASTS_KEY);
  if (!toasts) {
    throw new Error("No toasts context found. Did you forget to wrap your component with ToastProvider?");
  }
  return toasts;
}
function setToastsContext(toasts) {
  setContext(TOASTS_KEY, toasts);
}
function useToasts() {
  const toasts = getToastsContext();
  const removeToast = (index) => {
    toasts.update((toasts2) => {
      if (index < 0 || index >= toasts2.length) {
        return toasts2;
      }
      return toasts2.filter((_, i) => i !== index);
    });
  };
  const addToast = (toast) => {
    toasts.update((toasts2) => {
      const updatedToasts = [...toasts2, toast];
      return updatedToasts;
    });
  };
  const errToast = (message, detail) => {
    addToast({
      message,
      detail,
      type: "error",
      color: "red"
    });
  };
  return {
    toasts,
    addToast,
    removeToast,
    errToast
  };
}
export {
  setToastsContext as s,
  useToasts as u
};
//# sourceMappingURL=useToasts.js.map
