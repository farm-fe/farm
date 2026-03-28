import { type JsPlugin } from '@farmfe/core';
import { globby } from 'globby';
import { bold, green, yellow } from "colorette"
import fs, { type WriteFileOptions } from "fs-extra"
import path from "path"
import util from "util"
import type { Options as GlobbyOptions } from 'globby';


function stringify(value: any) {
  return util.inspect(value, { breakLength: Infinity })
}

function isObject(value: any) {
  return value !== null && typeof value === 'object'
}

async function isFile(filePath: string) {
  const fileStats = await fs.stat(filePath)

  return fileStats.isFile()
}

function renameTarget(target: string, rename: string | Function, src: string) {
  const parsedPath = path.parse(target)

  return typeof rename === 'string'
    ? rename
    : rename(parsedPath.name, parsedPath.ext.replace('.', ''), src)
}

async function generateCopyTarget(src: any, dest: any, { flatten, rename, transform }: any) {
  if (transform && !await isFile(src)) {
    throw new Error(`"transform" option works only on files: '${src}' must be a file`)
  }

  const { base, dir } = path.parse(src)
  const destinationFolder = (flatten || (!flatten && !dir))
    ? dest
    : dir.replace(dir.split('/')[0], dest)

  return {
    src,
    dest: path.join(destinationFolder, rename ? renameTarget(base, rename, src) : base),
    ...(transform && { contents: await transform(await fs.readFile(src), base) }),
    renamed: rename,
    transformed: transform
  }
}




interface Target extends GlobbyOptions {
  /**
   * Path or glob of what to copy.
   */
  readonly src: string | readonly string[];

  /**
   * One or more destinations where to copy.
   */
  readonly dest: string | readonly string[];

  /**
   * Change destination file or folder name.
   */
  readonly rename?: string | ((name: string, extension: string, fullPath: string) => string);

  /**
   * Modify file contents.
   */
  readonly transform?: (contents: Buffer, name: string) => any;
}

interface CopyOptions extends GlobbyOptions, fs.CopyOptions {
  /**
   * Copy items once. Useful in watch mode.
   * @default false
   */
  readonly copyOnce?: boolean;

  /**
   * Copy items synchronous.
   * @default false
   */
  readonly copySync?: boolean;

  /**
   * Remove the directory structure of copied files.
   * @default true
   */
  readonly flatten?: boolean;

  /**
   * Array of targets to copy.
   * @default []
   */
  readonly targets?: readonly Target[];

  /**
   * Output copied items to console.
   * @default false
   */
  readonly verbose?: boolean;
}


export default function farmPlugin(options: CopyOptions & WriteFileOptions): JsPlugin {


  const {
    copyOnce = false,
    copySync = false,
    flatten = true,
    targets = [],
    verbose = false,
    // @ts-ignore
    ...restPluginOptions
  } = options
  let copied = false;
  return {
    name: 'farm-plugin-copy',


    buildEnd: {
      executor: async () => {

        if (copyOnce && copied) {
          return
        }

        const copyTargets: any[] = []
        if (Array.isArray(targets) && targets.length) {
          for (const target of targets) {
            if (!isObject(target)) {
              throw new Error(`${stringify(target)} target must be an object`)
            }

            const { dest, rename, src, transform, ...restTargetOptions } = target

            if (!src || !dest) {
              throw new Error(`${stringify(target)} target must have "src" and "dest" properties`)
            }

            if (rename && typeof rename !== 'string' && typeof rename !== 'function') {
              throw new Error(`${stringify(target)} target's "rename" property must be a string or a function`)
            }

            const matchedPaths = await globby(src as (string | string[]), {
              expandDirectories: false,
              onlyFiles: false,
              ...restPluginOptions,
              ...restTargetOptions
            })

            if (matchedPaths.length) {
              for (const matchedPath of matchedPaths) {
                const generatedCopyTargets = Array.isArray(dest)
                  ? await Promise.all(dest.map((destination) => generateCopyTarget(
                    matchedPath,
                    destination,
                    { flatten, rename, transform }
                  )))
                  : [await generateCopyTarget(matchedPath, dest, { flatten, rename, transform })]

                copyTargets.push(...generatedCopyTargets)
              }
            }
          }
        }

        if (copyTargets.length) {
          if (verbose) {
            console.log(green('copied:'))
          }

          for (const copyTarget of copyTargets) {
            const { contents, dest, src, transformed } = copyTarget

            if (transformed) {
              await fs.outputFile(dest, contents, restPluginOptions)
            } else if (!copySync) {
              await fs.copy(src, dest, restPluginOptions)
            } else {
              fs.copySync(src, dest, restPluginOptions)
            }

            if (verbose) {
              let message = green(`  ${bold(src)} → ${bold(dest)}`)
              const flags = Object.entries(copyTarget)
                .filter(([key, value]) => ['renamed', 'transformed'].includes(key) && value)
                .map(([key]) => key.charAt(0).toUpperCase())

              if (flags.length) {
                message = `${message} ${yellow(`[${flags.join(', ')}]`)}`
              }

              console.log(message)
            }
          }
        } else if (verbose) {
          console.log(yellow('no items to copy'))
        }
        copied = true
      }
    }
  }
}

