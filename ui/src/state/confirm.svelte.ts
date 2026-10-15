/** Pregunta de confirmación abierta, si la hay. La pinta `components/ConfirmDialog.svelte`, montado una sola vez. */

export interface Confirmation {
  title: string;
  body: string;
  confirmLabel: string;
  /** Una segunda acción, además de confirmar y cancelar. */
  alternativeLabel?: string;
}

/** Lo que se contesta: la acción principal, la segunda o cancelar. */
export type Choice = "confirm" | "alternative" | "cancel";

interface Pending extends Confirmation {
  resolve: (choice: Choice) => void;
}

export const confirmation = $state<{ current: Pending | null }>({ current: null });

/** Pide confirmación en un diálogo modal. Se resuelve a `false` si se cancela o se pulsa Escape. */
export async function confirm(question: Confirmation): Promise<boolean> {
  return (await choose(question)) === "confirm";
}

/** Como `confirm`, pero con la segunda acción de `alternativeLabel`. Cancelar o Escape dan `"cancel"`. */
export function choose(question: Confirmation): Promise<Choice> {
  confirmation.current?.resolve("cancel");
  return new Promise((resolve) => (confirmation.current = { ...question, resolve }));
}
