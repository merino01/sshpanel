/** Pregunta de confirmación abierta, si la hay. La pinta `components/ConfirmDialog.svelte`, montado una sola vez. */

export interface Confirmation {
  title: string;
  body: string;
  confirmLabel: string;
}

interface Pending extends Confirmation {
  resolve: (confirmed: boolean) => void;
}

export const confirmation = $state<{ current: Pending | null }>({ current: null });

/** Pide confirmación en un diálogo modal. Se resuelve a `false` si se cancela o se pulsa Escape. */
export function confirm(question: Confirmation): Promise<boolean> {
  confirmation.current?.resolve(false);
  return new Promise((resolve) => (confirmation.current = { ...question, resolve }));
}
