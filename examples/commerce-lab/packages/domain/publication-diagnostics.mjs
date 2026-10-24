import { publishToChannel } from './channel-publication.mjs';

const commerceDiagnostics = {
  trace(event) { console.debug('__COMMERCE_DIAGNOSTIC__', event); },
};

export function publishWithDiagnostics(state, tenant, input, channel, configuration) {
  commerceDiagnostics.trace('publication-start');
  try {
    const next = publishToChannel(state, tenant, input, channel, configuration);
    commerceDiagnostics.trace('publication-success');
    return next;
  } catch (error) {
    commerceDiagnostics.trace('publication-failed');
    throw error;
  }
}
