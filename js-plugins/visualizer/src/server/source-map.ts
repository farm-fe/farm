import { SourceMapConsumer } from '@jridgewell/source-map';
import type { uint8 } from './interface';
import { byteToString } from './shared';

export interface ChunkMetadata {
  code: string;
  id: string;
}

export function pickupContentFromSourcemap(bytes: ArrayLike<uint8>) {
  const consumer = new SourceMapConsumer(byteToString(bytes), null);
  return consumer.sources.reduce((acc, cur) => {
    const content = consumer.sourceContentFor(cur, true);
    if (content) {
      acc.push({ id: cur, code: content });
    }
    return acc;
  }, [] as ChunkMetadata[]);
}

export function pickupMappingsFromCodeBinary(
  bytes: ArrayLike<uint8>,
  sourcemap: ArrayLike<uint8>,
  formatter: (id: string) => string
) {
  const consumer = new SourceMapConsumer(byteToString(sourcemap), null);
  const grouped: Record<string, string> = {};
  const files = new Set<string>();
  let line = 1;
  let column = 0;
  const code = byteToString(bytes);
  for (let i = 0; i < code.length; i++, column++) {
    const { source } = consumer.originalPositionFor({ line, column });
    if (source != null) {
      const id = formatter(source);

      const char = code[i];

      if (!(id in grouped)) {
        grouped[id] = '';
      }
      grouped[id] += char;
      files.add(id);
    }

    if (code[i] === '\n') {
      line += 1;
      column = -1;
    }
  }
  return { grouped, files };
}
