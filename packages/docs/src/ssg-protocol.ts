export type RenderTask =
  | { type: 'render'; id: number; key: string }
  | { type: 'renderNotFound'; id: number; locale: string };
export type RenderResponse =
  | { type: 'ready'; snapshotId: string }
  | { type: 'result'; id: number; html: string }
  | { type: 'error'; id: number; message: string }
  | {
      type: 'fatal';
      code: 'DOCS_RENDER_SNAPSHOT' | 'DOCS_RENDER_STARTUP';
      message: string;
    };
