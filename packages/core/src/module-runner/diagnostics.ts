import type {
  ModuleRunnerDiagnosticsEmitter,
  ModuleRunnerDiagnosticsEvent
} from './types.js';

export class ModuleRunnerDiagnosticsBus
  implements ModuleRunnerDiagnosticsEmitter
{
  private readonly listeners = new Set<
    (event: ModuleRunnerDiagnosticsEvent) => void
  >();

  emit(event: ModuleRunnerDiagnosticsEvent): void {
    for (const listener of this.listeners) {
      try {
        listener(event);
      } catch {
        // Diagnostics observers must not break the runner execution flow.
      }
    }
  }

  subscribe(
    listener: (event: ModuleRunnerDiagnosticsEvent) => void
  ): () => void {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  }

  clear(): void {
    this.listeners.clear();
  }
}
