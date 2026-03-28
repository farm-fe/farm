import { SourceLocation } from "@babel/types";

export enum ErrorSeverity {
  /**
   * Invalid JS syntax, or valid syntax that is semantically invalid which may indicate some
   * misunderstanding on the user’s part.
   */
  InvalidJS = "InvalidJS",
  /**
   * Code that breaks the rules of React.
   */
  InvalidReact = "InvalidReact",
  /**
   * Incorrect configuration of the compiler.
   */
  InvalidConfig = "InvalidConfig",
  /**
   * Code that can reasonably occur and that doesn't break any rules, but is unsafe to preserve
   * memoization.
   */
  CannotPreserveMemoization = "CannotPreserveMemoization",
  /**
   * Unhandled syntax that we don't support yet.
   */
  Todo = "Todo",
  /**
   * An unexpected internal error in the compiler that indicates critical issues that can panic
   * the compiler.
   */
  Invariant = "Invariant",
}

export type CompilerErrorDetailOptions = {
  reason: string;
  description?: string | null | undefined;
  severity: ErrorSeverity;
  loc: SourceLocation | null;
  suggestions?: Array<CompilerSuggestion> | null | undefined;
};

export enum CompilerSuggestionOperation {
  InsertBefore,
  InsertAfter,
  Remove,
  Replace,
}
export type CompilerSuggestion =
  | {
      op:
        | CompilerSuggestionOperation.InsertAfter
        | CompilerSuggestionOperation.InsertBefore
        | CompilerSuggestionOperation.Replace;
      range: [number, number];
      description: string;
      text: string;
    }
  | {
      op: CompilerSuggestionOperation.Remove;
      range: [number, number];
      description: string;
    };

/**
 * Represents 'events' that may occur during compilation. Events are only
 * recorded when a logger is set (through the config).
 * These are the different types of events:
 * CompileError:
 *   Forget skipped compilation of a function / file due to a known todo,
 *   invalid input, or compiler invariant being broken.
 * CompileSuccess:
 *   Forget successfully compiled a function.
 * PipelineError:
 *   Unexpected errors that occurred during compilation (e.g. failures in
 *   babel or other unhandled exceptions).
 */
export type LoggerEvent =
  | {
      kind: "CompileError";
      fnLoc: SourceLocation | null;
      detail: CompilerErrorDetailOptions;
    }
  | {
      kind: "CompileDiagnostic";
      fnLoc: SourceLocation | null;
      detail: Omit<Omit<CompilerErrorDetailOptions, "severity">, "suggestions">;
    }
  | {
      kind: "CompileSkip";
      fnLoc: SourceLocation | null;
      reason: string;
      loc: SourceLocation | null;
    }
  | {
      kind: "CompileSuccess";
      fnLoc: SourceLocation | null;
      fnName: string | null;
      memoSlots: number;
      memoBlocks: number;
      memoValues: number;
      prunedMemoBlocks: number;
      prunedMemoValues: number;
    }
  | {
      kind: "PipelineError";
      fnLoc: SourceLocation | null;
      data: string;
    };

export type Logger = {
  logEvent: (filename: string | null, event: LoggerEvent) => void;
};

export type ExternalFunction = {
  // Source for the imported module that exports the `importSpecifierName` functions
  source: string;

  // Unique name for the feature flag test condition, eg `isForgetEnabled_ProjectName`
  importSpecifierName: string;
};

type PanicThresholdOptions =
  /*
   * Any errors will panic the compiler by throwing an exception, which will
   * bubble up to the nearest exception handler above the Forget transform.
   * If Forget is invoked through `BabelPluginReactCompiler`, this will at the least
   * skip Forget compilation for the rest of current file.
   */
  | "all_errors"
  /*
   * Panic by throwing an exception only on critical or unrecognized errors.
   * For all other errors, skip the erroring function without inserting
   * a Forget-compiled version (i.e. same behavior as noEmit).
   */
  | "critical_errors"
  // Never panic by throwing an exception.
  | "none";
type CompilationMode =
  /*
   * Compiles functions annotated with "use forget" or component/hook-like functions.
   * This latter includes:
   * * Components declared with component syntax.
   * * Functions which can be inferred to be a component or hook:
   *   - Be named like a hook or component. This logic matches the ESLint rule.
   *   - *and* create JSX and/or call a hook. This is an additional check to help prevent
   *     false positives, since compilation has a greater impact than linting.
   * This is the default mode
   */
  | "infer"
  // Compile only components using Flow component syntax and hooks using hook syntax.
  | "syntax"
  // Compile only functions which are explicitly annotated with "use forget"
  | "annotation"
  // Compile all top-level functions
  | "all";

/**
 *
 * @link https://github.com/facebook/react/blob/main/compiler/packages/babel-plugin-react-compiler/src/Entrypoint/Options.ts#L39
 */
export type PluginOptions = {
  /**
   * @link https://github.com/facebook/react/blob/c80b336d23aa472b5e5910268138ac0447d6ae19/compiler/packages/babel-plugin-react-compiler/src/HIR/Environment.ts#L149
   */
  environment: object;

  logger: Logger | null;

  /*
   * Specifying a `gating` config, makes Forget compile and emit a separate
   * version of the function gated by importing the `gating.importSpecifierName` from the
   * specified `gating.source`.
   *
   * For example:
   *   gating: {
   *     source: 'ReactForgetFeatureFlag',
   *     importSpecifierName: 'isForgetEnabled_Pokes',
   *   }
   *
   * produces:
   *   import {isForgetEnabled_Pokes} from 'ReactForgetFeatureFlag';
   *
   *   Foo_forget()   {}
   *
   *   Foo_uncompiled() {}
   *
   *   var Foo = isForgetEnabled_Pokes() ? Foo_forget : Foo_uncompiled;
   */
  gating: ExternalFunction | null;

  panicThreshold: PanicThresholdOptions;

  /*
   * When enabled, Forget will continue statically analyzing and linting code, but skip over codegen
   * passes.
   *
   * Defaults to false
   */
  noEmit: boolean;

  /*
   * Determines the strategy for determining which functions to compile. Note that regardless of
   * which mode is enabled, a component can be opted out by adding the string literal
   * `"use no forget"` at the top of the function body, eg.:
   *
   * ```
   * function ComponentYouWantToSkipCompilation(props) {
   *    "use no forget";
   *    ...
   * }
   * ```
   */
  compilationMode: CompilationMode;

  /**
   * By default React Compiler will skip compilation of code that suppresses the default
   * React ESLint rules, since this is a strong indication that the code may be breaking React rules
   * in some way.
   *
   * Use eslintSuppressionRules to pass a custom set of rule names: any code which suppresses the
   * provided rules will skip compilation. To disable this feature (never bailout of compilation
   * even if the default ESLint is suppressed), pass an empty array.
   */
  eslintSuppressionRules?: Array<string> | null | undefined;

  flowSuppressions: boolean;
  /*
   * Ignore 'use no forget' annotations. Helpful during testing but should not be used in production.
   */
  ignoreUseNoForget: boolean;

  sources?: Array<string> | ((filename: string) => boolean) | null;

  /**
   * The compiler has customized support for react-native-reanimated, intended as a temporary workaround.
   * Set this flag (on by default) to automatically check for this library and activate the support.
   */
  enableReanimatedCheck: boolean;

  /**
   * The minimum major version of React that the compiler should emit code for. If the target is 19
   * or higher, the compiler emits direct imports of React runtime APIs needed by the compiler. On
   * versions prior to 19, an extra runtime package react-compiler-runtime is necessary to provide
   * a userspace approximation of runtime APIs.
   */
  target: CompilerReactTarget;
};

export type CompilerReactTarget =
  | "17"
  | "18"
  | "19"

  /**
   * Used exclusively for Meta apps which are guaranteed to have compatible
   * react runtime and compiler versions. Note that only the FB-internal bundles
   * re-export useMemoCache (see
   * https://github.com/facebook/react/blob/5b0ef217ef32333a8e56f39be04327c89efa346f/packages/react/index.fb.js#L68-L70),
   * so this option is invalid / creates runtime errors for open-source users.
   */
  | {
      kind: "donotuse_meta_internal";
      /**
       * @default react
       */
      runtimeModule: string;
    };
