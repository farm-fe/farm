import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
const __dirname = path.dirname(fileURLToPath(import.meta.url));
interface LanguageItem {
  hint?: string;
  message: string;
  invalidMessage?: string;
  dirForPrompts?: {
    current: string;
    target: string;
  };
  toggleOptions?: {
    active: string;
    inactive: string;
  };
  selectOptions?: {
    [key: string]: { title: string; desc?: string };
  };
}

interface Language {
  projectName: LanguageItem;
  shouldOverwrite: LanguageItem;
  packageName: LanguageItem;
  needsTypeScript: LanguageItem;
  needsRouter: LanguageItem;
  needsEslint: LanguageItem;
  needsPrettier: LanguageItem;
  needsSass: LanguageItem;
  useCssPreProcessor: LanguageItem;
  validTemplate: LanguageItem;
  selectFramework: LanguageItem;
  selectVariant: LanguageItem;
  errors: {
    operationCancelled: string;
  };
  defaultToggleOptions: {
    active: string;
    inactive: string;
  };
  copy: {
    scaffolding: string;
    done: string;
  };
  infos: {
    scaffolding: string;
    done: string;
  };
}

/**
 *
 * This function is used to link obtained locale with correct locale file in order to make locales reusable
 *
 * @param locale the obtained locale
 * @returns locale that linked with correct name
 */
function linkLocale(locale: string) {
  let linkedLocale: string;
  try {
    // @ts-ignore
    linkedLocale = Intl.getCanonicalLocales(locale)[0];
  } catch (error) {
    console.log(`${error.toString()}\n`);
  }
  switch (linkedLocale) {
    case 'zh-TW':
    case 'zh-HK':
    case 'zh-MO':
      linkedLocale = 'zh-Hant';
      break;
    case 'zh-CN':
    case 'zh-SG':
      linkedLocale = 'zh-Hans';
      break;
    default:
      linkedLocale = locale;
  }

  return linkedLocale;
}

function getLocale() {
  const shellLocale =
    process.env.LC_ALL || // POSIX locale environment variables
    process.env.LC_MESSAGES ||
    process.env.LANG ||
    Intl.DateTimeFormat().resolvedOptions().locale || // Built-in ECMA-402 support
    'en-US'; // Default fallback

  return linkLocale(shellLocale.split('.')[0].replace('_', '-'));
}

export function getLanguage() {
  const locale = getLocale();

  const localesRoot = path.resolve(__dirname, '../locales');
  console.log(localesRoot);

  const languageFilePath = path.resolve(localesRoot, `${locale}.json`);
  const doesLanguageExist = fs.existsSync(languageFilePath);

  const lang: Language = doesLanguageExist
    ? require(languageFilePath)
    : require(path.resolve(localesRoot, 'en-US.json'));

  return lang;
}
