import * as application from 'docs:application';
import { updateHead as head, BrowserRuntime as Runtime } from './browser-runtime-core.js';
import type { PublicPage } from './config.generated.js';
import type { PageLoader } from './page-loader-core.js';
export function BrowserRuntime(props: { initialKey: string | null; loader?: PageLoader }) {
  return <Runtime {...props} application={application} />;
}
export function updateHead(page: PublicPage) { head(page, application); }
