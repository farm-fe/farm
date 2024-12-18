export interface Module {
  id: string;
  exports?: any;
  // initialize promise if this module is a async module
  initializer?: Promise<any> | undefined;
  resource_pot?: string;
  meta?: Record<string, any>;
  require?: (id: string) => any;
}