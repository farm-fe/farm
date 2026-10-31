export interface DevelopmentTransaction {
  settle(): Promise<void>;
  isSettled(): boolean;
  validate(): Promise<void>;
  finalize(): void;
  restore(): unknown[];
}

export async function settleTogether(
  transactions: DevelopmentTransaction[]
): Promise<void> {
  do {
    const results = await Promise.allSettled(
      transactions.map((transaction) => transaction.settle())
    );
    const errors = results.flatMap((result) =>
      result.status === 'rejected' ? [result.reason] : []
    );
    if (errors.length)
      throw new AggregateError(
        errors,
        'Development transaction settling failed'
      );
  } while (!transactions.every((transaction) => transaction.isSettled()));
}

export async function validateTogether(
  transactions: DevelopmentTransaction[]
): Promise<void> {
  do {
    const results = await Promise.allSettled(
      transactions.map((transaction) => transaction.validate())
    );
    const errors = results.flatMap((result) =>
      result.status === 'rejected' ? [result.reason] : []
    );
    if (errors.length)
      throw new AggregateError(
        errors,
        'Development transaction validation failed'
      );
  } while (!transactions.every((transaction) => transaction.isSettled()));
}

export async function restoreTogether(
  transactions: DevelopmentTransaction[]
): Promise<unknown[]> {
  await settleTogether(transactions);
  const diagnostics: unknown[] = [],
    failures: unknown[] = [];
  // No await between restorations: a candidate script cannot run between registry restores.
  for (const transaction of [...transactions].reverse()) {
    try {
      diagnostics.push(...transaction.restore());
    } catch (error) {
      failures.push(error);
    }
  }
  if (failures.length)
    throw new AggregateError(
      failures,
      'Development transaction restoration failed'
    );
  return diagnostics;
}
