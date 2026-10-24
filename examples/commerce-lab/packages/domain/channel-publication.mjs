import { createCampaign } from './index.mjs';

/** Public build capabilities supplement, rather than replace, merchant policy. */
export function publishToChannel(state, tenant, input, channel, configuration) {
  if (!configuration || typeof configuration.version !== 'string' || !configuration.version.trim() ||
      !Array.isArray(configuration.channels) || !configuration.channels.length ||
      configuration.channels.some(value => !['Store', 'Social', 'Marketplace'].includes(value))) {
    throw new Error('Invalid channel configuration');
  }
  if (!configuration.channels.includes(channel)) throw new Error('Unsupported publication channel');
  return createCampaign(state, tenant, {
    ...input,
    channel,
    channelRuleVersion: configuration.version,
  });
}
