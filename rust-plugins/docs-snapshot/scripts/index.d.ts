export interface Verification {
  browserReceipt: string;
  browserHash: string;
  serverReceipt: string;
  serverHash: string;
  writesPath: string;
  writesHash: string;
  publicDirectory: string;
  serverDirectory: string;
  templatePath: string;
}
export type SnapshotOptions = {
  mode: 'capture';
  planPath: string;
  receiptPath: string;
  seedPath: string;
  target: 'browser' | 'server';
  outputPath: string;
} | {
  mode: 'verify';
  planPath: string;
  verification: Verification;
};
export default function snapshot(options: SnapshotOptions): [string, SnapshotOptions];
