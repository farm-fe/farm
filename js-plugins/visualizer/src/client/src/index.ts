// import './style.css';
// import 'ant-design-vue/dist/reset.css';

// import { install as VueMonacoEditorPlugin } from '@guolao/vue-monaco-editor';
// import { createPinia } from 'pinia';
// // register vue composition api globally
// import { createApp } from 'vue';
// import App from './App.vue';
// import router from './router';

// const app = createApp(App);
// const pinia = createPinia();
// app.use(pinia);
// app.use(VueMonacoEditorPlugin, {
//   paths: {
//     // The recommended CDN config
//     vs: 'https://cdn.jsdelivr.net/npm/monaco-editor@0.43.0/min/vs'
//   }
// });

// app.use(router);
// app.mount('#app');

import { AnalyzerClient } from 'vite-bundle-analyzer/sdk/browser';
import 'vite-bundle-analyzer/sdk/browser.css';

const data = [
  {
    filename: 'assets/index-T79nKnwS.css',
    label: 'assets/index-T79nKnwS.css',
    parsedSize: 12975,
    mapSize: 0,
    statSize: 12975,
    gzipSize: 3159,
    brotliSize: 2683,
    source: [
      {
        parsedSize: 12975,
        gzipSize: 3159,
        brotliSize: 2683,
        label: 'assets/index-T79nKnwS.css',
        filename: 'assets/index-T79nKnwS.css'
      }
    ],
    stats: [
      {
        statSize: 12975,
        label: 'assets/index-T79nKnwS.css',
        filename: 'assets/index-T79nKnwS.css'
      }
    ],
    isAsset: true,
    isEntry: false,
    imports: []
  },
  {
    filename: 'assets/index-bBk9ut-R.js',
    label: 'assets/index-bBk9ut-R.js',
    parsedSize: 96305,
    mapSize: 354379,
    statSize: 188149,
    gzipSize: 41567,
    brotliSize: 36759,
    source: [
      {
        parsedSize: 96305,
        gzipSize: 41567,
        brotliSize: 36759,
        label: 'Users',
        groups: [
          {
            parsedSize: 44665,
            gzipSize: 21948,
            brotliSize: 19085,
            label: 'src',
            groups: [
              {
                parsedSize: 44500,
                gzipSize: 21792,
                brotliSize: 18964,
                label: 'client',
                groups: [
                  {
                    parsedSize: 38455,
                    gzipSize: 18180,
                    brotliSize: 15949,
                    label: 'components',
                    groups: [
                      {
                        parsedSize: 2186,
                        gzipSize: 1141,
                        brotliSize: 1030,
                        label: 'side-bar',
                        groups: [
                          {
                            parsedSize: 131,
                            gzipSize: 118,
                            brotliSize: 117,
                            label: 'provide.ts',
                            filename:
                              'Users/src/client/components/side-bar/provide.ts'
                          },
                          {
                            parsedSize: 2055,
                            gzipSize: 1023,
                            brotliSize: 913,
                            label: 'side-bar.tsx',
                            filename:
                              'Users/src/client/components/side-bar/side-bar.tsx'
                          }
                        ],
                        filename: 'Users/src/client/components/side-bar'
                      },
                      {
                        parsedSize: 5128,
                        gzipSize: 2447,
                        brotliSize: 2126,
                        label: 'drawer',
                        groups: [
                          {
                            parsedSize: 738,
                            gzipSize: 392,
                            brotliSize: 333,
                            label: 'content.tsx',
                            filename:
                              'Users/src/client/components/drawer/content.tsx'
                          },
                          {
                            parsedSize: 1232,
                            gzipSize: 685,
                            brotliSize: 605,
                            label: 'backdrop.tsx',
                            filename:
                              'Users/src/client/components/drawer/backdrop.tsx'
                          },
                          {
                            parsedSize: 2827,
                            gzipSize: 1086,
                            brotliSize: 951,
                            label: 'wrapper.tsx',
                            filename:
                              'Users/src/client/components/drawer/wrapper.tsx'
                          },
                          {
                            parsedSize: 305,
                            gzipSize: 238,
                            brotliSize: 207,
                            label: 'drawer.tsx',
                            filename:
                              'Users/src/client/components/drawer/drawer.tsx'
                          },
                          {
                            parsedSize: 26,
                            gzipSize: 46,
                            brotliSize: 30,
                            label: 'index.ts',
                            filename:
                              'Users/src/client/components/drawer/index.ts'
                          }
                        ],
                        filename: 'Users/src/client/components/drawer'
                      },
                      {
                        parsedSize: 4525,
                        gzipSize: 2178,
                        brotliSize: 1917,
                        label: 'checkbox',
                        groups: [
                          {
                            parsedSize: 81,
                            gzipSize: 99,
                            brotliSize: 74,
                            label: 'context.ts',
                            filename:
                              'Users/src/client/components/checkbox/context.ts'
                          },
                          {
                            parsedSize: 2882,
                            gzipSize: 1365,
                            brotliSize: 1221,
                            label: 'checkbox.tsx',
                            filename:
                              'Users/src/client/components/checkbox/checkbox.tsx'
                          },
                          {
                            parsedSize: 1535,
                            gzipSize: 667,
                            brotliSize: 591,
                            label: 'checkbox-group.tsx',
                            filename:
                              'Users/src/client/components/checkbox/checkbox-group.tsx'
                          },
                          {
                            parsedSize: 27,
                            gzipSize: 47,
                            brotliSize: 31,
                            label: 'index.ts',
                            filename:
                              'Users/src/client/components/checkbox/index.ts'
                          }
                        ],
                        filename: 'Users/src/client/components/checkbox'
                      },
                      {
                        parsedSize: 2600,
                        gzipSize: 1132,
                        brotliSize: 987,
                        label: 'text',
                        groups: [
                          {
                            parsedSize: 2058,
                            gzipSize: 803,
                            brotliSize: 702,
                            label: 'child.tsx',
                            filename:
                              'Users/src/client/components/text/child.tsx'
                          },
                          {
                            parsedSize: 542,
                            gzipSize: 329,
                            brotliSize: 285,
                            label: 'text.tsx',
                            filename:
                              'Users/src/client/components/text/text.tsx'
                          }
                        ],
                        filename: 'Users/src/client/components/text'
                      },
                      {
                        parsedSize: 400,
                        gzipSize: 266,
                        brotliSize: 227,
                        label: 'module-item.tsx',
                        filename: 'Users/src/client/components/module-item.tsx'
                      },
                      {
                        parsedSize: 1369,
                        gzipSize: 713,
                        brotliSize: 638,
                        label: 'file-list.tsx',
                        filename: 'Users/src/client/components/file-list.tsx'
                      },
                      {
                        parsedSize: 4554,
                        gzipSize: 1615,
                        brotliSize: 1457,
                        label: 'input',
                        groups: [
                          {
                            parsedSize: 4554,
                            gzipSize: 1615,
                            brotliSize: 1457,
                            label: 'input.tsx',
                            filename:
                              'Users/src/client/components/input/input.tsx'
                          }
                        ],
                        filename: 'Users/src/client/components/input'
                      },
                      {
                        parsedSize: 1283,
                        gzipSize: 667,
                        brotliSize: 582,
                        label: 'search-modules.tsx',
                        filename:
                          'Users/src/client/components/search-modules.tsx'
                      },
                      {
                        parsedSize: 9325,
                        gzipSize: 4848,
                        brotliSize: 4204,
                        label: 'select',
                        groups: [
                          {
                            parsedSize: 115,
                            gzipSize: 124,
                            brotliSize: 97,
                            label: 'context.ts',
                            filename:
                              'Users/src/client/components/select/context.ts'
                          },
                          {
                            parsedSize: 367,
                            gzipSize: 231,
                            brotliSize: 190,
                            label: 'layouts.ts',
                            filename:
                              'Users/src/client/components/select/layouts.ts'
                          },
                          {
                            parsedSize: 1389,
                            gzipSize: 775,
                            brotliSize: 672,
                            label: 'dropdown.tsx',
                            filename:
                              'Users/src/client/components/select/dropdown.tsx'
                          },
                          {
                            parsedSize: 79,
                            gzipSize: 91,
                            brotliSize: 71,
                            label: 'ellipsis.tsx',
                            filename:
                              'Users/src/client/components/select/ellipsis.tsx'
                          },
                          {
                            parsedSize: 1245,
                            gzipSize: 708,
                            brotliSize: 608,
                            label: 'select-multiple.tsx',
                            filename:
                              'Users/src/client/components/select/select-multiple.tsx'
                          },
                          {
                            parsedSize: 2654,
                            gzipSize: 1136,
                            brotliSize: 981,
                            label: 'select-option.tsx',
                            filename:
                              'Users/src/client/components/select/select-option.tsx'
                          },
                          {
                            parsedSize: 3453,
                            gzipSize: 1740,
                            brotliSize: 1558,
                            label: 'select.tsx',
                            filename:
                              'Users/src/client/components/select/select.tsx'
                          },
                          {
                            parsedSize: 23,
                            gzipSize: 43,
                            brotliSize: 27,
                            label: 'index.ts',
                            filename:
                              'Users/src/client/components/select/index.ts'
                          }
                        ],
                        filename: 'Users/src/client/components/select'
                      },
                      {
                        parsedSize: 867,
                        gzipSize: 545,
                        brotliSize: 468,
                        label: 'tooltip.tsx',
                        filename: 'Users/src/client/components/tooltip.tsx'
                      },
                      {
                        parsedSize: 888,
                        gzipSize: 519,
                        brotliSize: 470,
                        label: 'treemap',
                        groups: [
                          {
                            parsedSize: 888,
                            gzipSize: 519,
                            brotliSize: 470,
                            label: 'component.tsx',
                            filename:
                              'Users/src/client/components/treemap/component.tsx'
                          }
                        ],
                        filename: 'Users/src/client/components/treemap'
                      },
                      {
                        parsedSize: 3918,
                        gzipSize: 1374,
                        brotliSize: 1208,
                        label: 'button/button.tsx',
                        filename:
                          'Users/src/client/components/button/button.tsx'
                      },
                      {
                        parsedSize: 497,
                        gzipSize: 308,
                        brotliSize: 265,
                        label: 'css-transition/css-transition.ts',
                        filename:
                          'Users/src/client/components/css-transition/css-transition.ts'
                      },
                      {
                        parsedSize: 915,
                        gzipSize: 427,
                        brotliSize: 370,
                        label: 'spacer/spacer.tsx',
                        filename:
                          'Users/src/client/components/spacer/spacer.tsx'
                      }
                    ],
                    filename: 'Users/src/client/components'
                  },
                  {
                    parsedSize: 563,
                    gzipSize: 296,
                    brotliSize: 269,
                    label: 'context.ts',
                    filename: 'Users/src/client/context.ts'
                  },
                  {
                    parsedSize: 91,
                    gzipSize: 111,
                    brotliSize: 93,
                    label: 'special',
                    groups: [
                      {
                        parsedSize: 91,
                        gzipSize: 111,
                        brotliSize: 93,
                        label: 'index.ts',
                        filename: 'Users/src/client/special/index.ts'
                      }
                    ],
                    filename: 'Users/src/client/special'
                  },
                  {
                    parsedSize: 3762,
                    gzipSize: 2285,
                    brotliSize: 1881,
                    label: 'composables',
                    groups: [
                      {
                        parsedSize: 895,
                        gzipSize: 453,
                        brotliSize: 381,
                        label: 'use-body-scroll',
                        groups: [
                          {
                            parsedSize: 895,
                            gzipSize: 453,
                            brotliSize: 381,
                            label: 'use-body-scroll.ts',
                            filename:
                              'Users/src/client/composables/use-body-scroll/use-body-scroll.ts'
                          }
                        ],
                        filename: 'Users/src/client/composables/use-body-scroll'
                      },
                      {
                        parsedSize: 213,
                        gzipSize: 178,
                        brotliSize: 154,
                        label: 'use-dom-observer',
                        groups: [
                          {
                            parsedSize: 213,
                            gzipSize: 178,
                            brotliSize: 154,
                            label: 'use-dom-observer.ts',
                            filename:
                              'Users/src/client/composables/use-dom-observer/use-dom-observer.ts'
                          }
                        ],
                        filename:
                          'Users/src/client/composables/use-dom-observer'
                      },
                      {
                        parsedSize: 273,
                        gzipSize: 220,
                        brotliSize: 156,
                        label: 'use-portal',
                        groups: [
                          {
                            parsedSize: 273,
                            gzipSize: 220,
                            brotliSize: 156,
                            label: 'use-portal.ts',
                            filename:
                              'Users/src/client/composables/use-portal/use-portal.ts'
                          }
                        ],
                        filename: 'Users/src/client/composables/use-portal'
                      },
                      {
                        parsedSize: 157,
                        gzipSize: 132,
                        brotliSize: 104,
                        label: 'use-resize',
                        groups: [
                          {
                            parsedSize: 157,
                            gzipSize: 132,
                            brotliSize: 104,
                            label: 'use-resize.ts',
                            filename:
                              'Users/src/client/composables/use-resize/use-resize.ts'
                          }
                        ],
                        filename: 'Users/src/client/composables/use-resize'
                      },
                      {
                        parsedSize: 1713,
                        gzipSize: 1034,
                        brotliSize: 889,
                        label: 'use-scale',
                        groups: [
                          {
                            parsedSize: 491,
                            gzipSize: 291,
                            brotliSize: 245,
                            label: 'scale-context.ts',
                            filename:
                              'Users/src/client/composables/use-scale/scale-context.ts'
                          },
                          {
                            parsedSize: 188,
                            gzipSize: 148,
                            brotliSize: 124,
                            label: 'utils.ts',
                            filename:
                              'Users/src/client/composables/use-scale/utils.ts'
                          },
                          {
                            parsedSize: 1034,
                            gzipSize: 595,
                            brotliSize: 520,
                            label: 'with-scale.tsx',
                            filename:
                              'Users/src/client/composables/use-scale/with-scale.tsx'
                          }
                        ],
                        filename: 'Users/src/client/composables/use-scale'
                      },
                      {
                        parsedSize: 97,
                        gzipSize: 86,
                        brotliSize: 64,
                        label: 'use-click-anywhere',
                        groups: [
                          {
                            parsedSize: 97,
                            gzipSize: 86,
                            brotliSize: 64,
                            label: 'use-click-anywhere.ts',
                            filename:
                              'Users/src/client/composables/use-click-anywhere/use-click-anywhere.ts'
                          }
                        ],
                        filename:
                          'Users/src/client/composables/use-click-anywhere'
                      },
                      {
                        parsedSize: 414,
                        gzipSize: 182,
                        brotliSize: 133,
                        label: 'use-query',
                        groups: [
                          {
                            parsedSize: 414,
                            gzipSize: 182,
                            brotliSize: 133,
                            label: 'use-query.ts',
                            filename:
                              'Users/src/client/composables/use-query/use-query.ts'
                          }
                        ],
                        filename: 'Users/src/client/composables/use-query'
                      }
                    ],
                    filename: 'Users/src/client/composables'
                  },
                  {
                    parsedSize: 139,
                    gzipSize: 125,
                    brotliSize: 96,
                    label: 'shared.ts',
                    filename: 'Users/src/client/shared.ts'
                  },
                  {
                    parsedSize: 411,
                    gzipSize: 228,
                    brotliSize: 183,
                    label: 'receiver.tsx',
                    filename: 'Users/src/client/receiver.tsx'
                  },
                  {
                    parsedSize: 955,
                    gzipSize: 440,
                    brotliSize: 388,
                    label: 'application.tsx',
                    filename: 'Users/src/client/application.tsx'
                  },
                  {
                    parsedSize: 124,
                    gzipSize: 127,
                    brotliSize: 105,
                    label: 'main.tsx',
                    filename: 'Users/src/client/main.tsx'
                  }
                ],
                filename: 'Users/src/client'
              },
              {
                parsedSize: 165,
                gzipSize: 156,
                brotliSize: 121,
                label: 'shared/index.ts',
                filename: 'Users/src/shared/index.ts'
              }
            ],
            filename: 'Users/src'
          },
          {
            parsedSize: 51640,
            gzipSize: 19619,
            brotliSize: 17674,
            label: 'node_modules/.pnpm',
            groups: [
              {
                parsedSize: 25601,
                gzipSize: 8310,
                brotliSize: 7466,
                label:
                  'squarified@0.3.2/node_modules/squarified/dist/index.mjs',
                filename:
                  'Users/node_modules/.pnpm/squarified@0.3.2/node_modules/squarified/dist/index.mjs'
              },
              {
                parsedSize: 23886,
                gzipSize: 9912,
                brotliSize: 9034,
                label: 'preact@10.22.0/node_modules/preact',
                groups: [
                  {
                    parsedSize: 9100,
                    gzipSize: 3723,
                    brotliSize: 3410,
                    label: 'compat',
                    groups: [
                      {
                        parsedSize: 142,
                        gzipSize: 116,
                        brotliSize: 99,
                        label: 'client.mjs',
                        filename:
                          'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/compat/client.mjs'
                      },
                      {
                        parsedSize: 8958,
                        gzipSize: 3607,
                        brotliSize: 3311,
                        label: 'dist/compat.module.js',
                        filename:
                          'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/compat/dist/compat.module.js'
                      }
                    ],
                    filename:
                      'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/compat'
                  },
                  {
                    parsedSize: 11053,
                    gzipSize: 4562,
                    brotliSize: 4149,
                    label: 'dist/preact.module.js',
                    filename:
                      'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/dist/preact.module.js'
                  },
                  {
                    parsedSize: 3367,
                    gzipSize: 1370,
                    brotliSize: 1253,
                    label: 'hooks/dist',
                    groups: [
                      {
                        parsedSize: 3367,
                        gzipSize: 1370,
                        brotliSize: 1253,
                        label: 'hooks.module.js',
                        filename:
                          'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/hooks/dist/hooks.module.js'
                      }
                    ],
                    filename:
                      'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/hooks/dist'
                  },
                  {
                    parsedSize: 366,
                    gzipSize: 257,
                    brotliSize: 222,
                    label: 'jsx-runtime/dist',
                    groups: [
                      {
                        parsedSize: 366,
                        gzipSize: 257,
                        brotliSize: 222,
                        label: 'jsxRuntime.module.js',
                        filename:
                          'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/jsx-runtime/dist/jsxRuntime.module.js'
                      }
                    ],
                    filename:
                      'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact/jsx-runtime/dist'
                  }
                ],
                filename:
                  'Users/node_modules/.pnpm/preact@10.22.0/node_modules/preact'
              },
              {
                parsedSize: 1413,
                gzipSize: 786,
                brotliSize: 700,
                label:
                  '@stylexjs+stylex@0.9.3/node_modules/@stylexjs/stylex/lib/es/stylex.mjs',
                filename:
                  'Users/node_modules/.pnpm/@stylexjs+stylex@0.9.3/node_modules/@stylexjs/stylex/lib/es/stylex.mjs'
              },
              {
                parsedSize: 381,
                gzipSize: 376,
                brotliSize: 286,
                label: 'foxact@0.2.35_react@18.3.1/node_modules/foxact',
                groups: [
                  {
                    parsedSize: 97,
                    gzipSize: 102,
                    brotliSize: 78,
                    label: 'compose-context-provider/index.mjs',
                    filename:
                      'Users/node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/compose-context-provider/index.mjs'
                  },
                  {
                    parsedSize: 10,
                    gzipSize: 30,
                    brotliSize: 14,
                    label: 'noop/index.mjs',
                    filename:
                      'Users/node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/noop/index.mjs'
                  },
                  {
                    parsedSize: 175,
                    gzipSize: 138,
                    brotliSize: 110,
                    label: 'context-state/index.mjs',
                    filename:
                      'Users/node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/context-state/index.mjs'
                  },
                  {
                    parsedSize: 99,
                    gzipSize: 106,
                    brotliSize: 84,
                    label: 'use-abortable-effect/index.mjs',
                    filename:
                      'Users/node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/use-abortable-effect/index.mjs'
                  }
                ],
                filename:
                  'Users/node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact'
              },
              {
                parsedSize: 359,
                gzipSize: 235,
                brotliSize: 188,
                label: 'clsx@2.1.1/node_modules/clsx/dist/clsx.mjs',
                filename:
                  'Users/node_modules/.pnpm/clsx@2.1.1/node_modules/clsx/dist/clsx.mjs'
              }
            ],
            filename: 'Users/node_modules/.pnpm'
          }
        ],
        filename: 'Users'
      }
    ],
    stats: [
      {
        statSize: 98731,
        label: 'src',
        groups: [
          {
            statSize: 98329,
            label: 'client',
            groups: [
              {
                statSize: 80807,
                label: 'components',
                groups: [
                  {
                    statSize: 6552,
                    label: 'side-bar',
                    groups: [
                      {
                        statSize: 525,
                        label: 'provide.ts',
                        filename: 'src/client/components/side-bar/provide.ts'
                      },
                      {
                        statSize: 6027,
                        label: 'side-bar.tsx',
                        filename: 'src/client/components/side-bar/side-bar.tsx'
                      }
                    ],
                    filename: 'src/client/components/side-bar'
                  },
                  {
                    statSize: 9948,
                    label: 'drawer',
                    groups: [
                      {
                        statSize: 1621,
                        label: 'content.tsx',
                        filename: 'src/client/components/drawer/content.tsx'
                      },
                      {
                        statSize: 2492,
                        label: 'backdrop.tsx',
                        filename: 'src/client/components/drawer/backdrop.tsx'
                      },
                      {
                        statSize: 4483,
                        label: 'wrapper.tsx',
                        filename: 'src/client/components/drawer/wrapper.tsx'
                      },
                      {
                        statSize: 1200,
                        label: 'drawer.tsx',
                        filename: 'src/client/components/drawer/drawer.tsx'
                      },
                      {
                        statSize: 152,
                        label: 'index.ts',
                        filename: 'src/client/components/drawer/index.ts'
                      }
                    ],
                    filename: 'src/client/components/drawer'
                  },
                  {
                    statSize: 9428,
                    label: 'checkbox',
                    groups: [
                      {
                        statSize: 376,
                        label: 'context.ts',
                        filename: 'src/client/components/checkbox/context.ts'
                      },
                      {
                        statSize: 5692,
                        label: 'checkbox.tsx',
                        filename: 'src/client/components/checkbox/checkbox.tsx'
                      },
                      {
                        statSize: 3191,
                        label: 'checkbox-group.tsx',
                        filename:
                          'src/client/components/checkbox/checkbox-group.tsx'
                      },
                      {
                        statSize: 169,
                        label: 'index.ts',
                        filename: 'src/client/components/checkbox/index.ts'
                      }
                    ],
                    filename: 'src/client/components/checkbox'
                  },
                  {
                    statSize: 5730,
                    label: 'text',
                    groups: [
                      {
                        statSize: 4172,
                        label: 'child.tsx',
                        filename: 'src/client/components/text/child.tsx'
                      },
                      {
                        statSize: 1558,
                        label: 'text.tsx',
                        filename: 'src/client/components/text/text.tsx'
                      }
                    ],
                    filename: 'src/client/components/text'
                  },
                  {
                    statSize: 1074,
                    label: 'module-item.tsx',
                    filename: 'src/client/components/module-item.tsx'
                  },
                  {
                    statSize: 1854,
                    label: 'file-list.tsx',
                    filename: 'src/client/components/file-list.tsx'
                  },
                  {
                    statSize: 8145,
                    label: 'input',
                    groups: [
                      {
                        statSize: 8145,
                        label: 'input.tsx',
                        filename: 'src/client/components/input/input.tsx'
                      }
                    ],
                    filename: 'src/client/components/input'
                  },
                  {
                    statSize: 3846,
                    label: 'search-modules.tsx',
                    filename: 'src/client/components/search-modules.tsx'
                  },
                  {
                    statSize: 20199,
                    label: 'select',
                    groups: [
                      {
                        statSize: 270,
                        label: 'context.ts',
                        filename: 'src/client/components/select/context.ts'
                      },
                      {
                        statSize: 1190,
                        label: 'layouts.ts',
                        filename: 'src/client/components/select/layouts.ts'
                      },
                      {
                        statSize: 3085,
                        label: 'dropdown.tsx',
                        filename: 'src/client/components/select/dropdown.tsx'
                      },
                      {
                        statSize: 602,
                        label: 'ellipsis.tsx',
                        filename: 'src/client/components/select/ellipsis.tsx'
                      },
                      {
                        statSize: 2130,
                        label: 'select-multiple.tsx',
                        filename:
                          'src/client/components/select/select-multiple.tsx'
                      },
                      {
                        statSize: 5149,
                        label: 'select-option.tsx',
                        filename:
                          'src/client/components/select/select-option.tsx'
                      },
                      {
                        statSize: 7618,
                        label: 'select.tsx',
                        filename: 'src/client/components/select/select.tsx'
                      },
                      {
                        statSize: 155,
                        label: 'index.ts',
                        filename: 'src/client/components/select/index.ts'
                      }
                    ],
                    filename: 'src/client/components/select'
                  },
                  {
                    statSize: 1890,
                    label: 'tooltip.tsx',
                    filename: 'src/client/components/tooltip.tsx'
                  },
                  {
                    statSize: 2454,
                    label: 'treemap',
                    groups: [
                      {
                        statSize: 2454,
                        label: 'component.tsx',
                        filename: 'src/client/components/treemap/component.tsx'
                      }
                    ],
                    filename: 'src/client/components/treemap'
                  },
                  {
                    statSize: 6431,
                    label: 'button/button.tsx',
                    filename: 'src/client/components/button/button.tsx'
                  },
                  {
                    statSize: 1438,
                    label: 'css-transition/css-transition.ts',
                    filename:
                      'src/client/components/css-transition/css-transition.ts'
                  },
                  {
                    statSize: 1818,
                    label: 'spacer/spacer.tsx',
                    filename: 'src/client/components/spacer/spacer.tsx'
                  }
                ],
                filename: 'src/client/components'
              },
              {
                statSize: 1718,
                label: 'context.ts',
                filename: 'src/client/context.ts'
              },
              {
                statSize: 875,
                label: 'shared.ts',
                filename: 'src/client/shared.ts'
              },
              {
                statSize: 264,
                label: 'special',
                groups: [
                  {
                    statSize: 264,
                    label: 'index.ts',
                    filename: 'src/client/special/index.ts'
                  }
                ],
                filename: 'src/client/special'
              },
              {
                statSize: 9783,
                label: 'composables',
                groups: [
                  {
                    statSize: 1899,
                    label: 'use-body-scroll',
                    groups: [
                      {
                        statSize: 1899,
                        label: 'use-body-scroll.ts',
                        filename:
                          'src/client/composables/use-body-scroll/use-body-scroll.ts'
                      }
                    ],
                    filename: 'src/client/composables/use-body-scroll'
                  },
                  {
                    statSize: 231,
                    label: 'use-click-anywhere',
                    groups: [
                      {
                        statSize: 231,
                        label: 'use-click-anywhere.ts',
                        filename:
                          'src/client/composables/use-click-anywhere/use-click-anywhere.ts'
                      }
                    ],
                    filename: 'src/client/composables/use-click-anywhere'
                  },
                  {
                    statSize: 559,
                    label: 'use-dom-observer',
                    groups: [
                      {
                        statSize: 559,
                        label: 'use-dom-observer.ts',
                        filename:
                          'src/client/composables/use-dom-observer/use-dom-observer.ts'
                      }
                    ],
                    filename: 'src/client/composables/use-dom-observer'
                  },
                  {
                    statSize: 729,
                    label: 'use-portal',
                    groups: [
                      {
                        statSize: 729,
                        label: 'use-portal.ts',
                        filename:
                          'src/client/composables/use-portal/use-portal.ts'
                      }
                    ],
                    filename: 'src/client/composables/use-portal'
                  },
                  {
                    statSize: 743,
                    label: 'use-query',
                    groups: [
                      {
                        statSize: 743,
                        label: 'use-query.ts',
                        filename:
                          'src/client/composables/use-query/use-query.ts'
                      }
                    ],
                    filename: 'src/client/composables/use-query'
                  },
                  {
                    statSize: 406,
                    label: 'use-resize',
                    groups: [
                      {
                        statSize: 406,
                        label: 'use-resize.ts',
                        filename:
                          'src/client/composables/use-resize/use-resize.ts'
                      }
                    ],
                    filename: 'src/client/composables/use-resize'
                  },
                  {
                    statSize: 5216,
                    label: 'use-scale',
                    groups: [
                      {
                        statSize: 1209,
                        label: 'scale-context.ts',
                        filename:
                          'src/client/composables/use-scale/scale-context.ts'
                      },
                      {
                        statSize: 781,
                        label: 'utils.ts',
                        filename: 'src/client/composables/use-scale/utils.ts'
                      },
                      {
                        statSize: 3226,
                        label: 'with-scale.tsx',
                        filename:
                          'src/client/composables/use-scale/with-scale.tsx'
                      }
                    ],
                    filename: 'src/client/composables/use-scale'
                  }
                ],
                filename: 'src/client/composables'
              },
              {
                statSize: 1043,
                label: 'receiver.tsx',
                filename: 'src/client/receiver.tsx'
              },
              {
                statSize: 3437,
                label: 'application.tsx',
                filename: 'src/client/application.tsx'
              },
              {
                statSize: 402,
                label: 'main.tsx',
                filename: 'src/client/main.tsx'
              }
            ],
            filename: 'src/client'
          },
          {
            statSize: 402,
            label: 'shared/index.ts',
            filename: 'src/shared/index.ts'
          }
        ],
        filename: 'src'
      },
      {
        statSize: 89418,
        label: 'node_modules/.pnpm',
        groups: [
          {
            statSize: 27492,
            label: 'preact@10.22.0/node_modules/preact',
            groups: [
              {
                statSize: 10610,
                label: 'compat',
                groups: [
                  {
                    statSize: 489,
                    label: 'client.mjs',
                    filename:
                      'node_modules/.pnpm/preact@10.22.0/node_modules/preact/compat/client.mjs'
                  },
                  {
                    statSize: 10121,
                    label: 'dist/compat.module.js',
                    filename:
                      'node_modules/.pnpm/preact@10.22.0/node_modules/preact/compat/dist/compat.module.js'
                  }
                ],
                filename:
                  'node_modules/.pnpm/preact@10.22.0/node_modules/preact/compat'
              },
              {
                statSize: 11366,
                label: 'dist/preact.module.js',
                filename:
                  'node_modules/.pnpm/preact@10.22.0/node_modules/preact/dist/preact.module.js'
              },
              {
                statSize: 3783,
                label: 'hooks/dist',
                groups: [
                  {
                    statSize: 3783,
                    label: 'hooks.module.js',
                    filename:
                      'node_modules/.pnpm/preact@10.22.0/node_modules/preact/hooks/dist/hooks.module.js'
                  }
                ],
                filename:
                  'node_modules/.pnpm/preact@10.22.0/node_modules/preact/hooks/dist'
              },
              {
                statSize: 1733,
                label: 'jsx-runtime/dist',
                groups: [
                  {
                    statSize: 1733,
                    label: 'jsxRuntime.module.js',
                    filename:
                      'node_modules/.pnpm/preact@10.22.0/node_modules/preact/jsx-runtime/dist/jsxRuntime.module.js'
                  }
                ],
                filename:
                  'node_modules/.pnpm/preact@10.22.0/node_modules/preact/jsx-runtime/dist'
              }
            ],
            filename: 'node_modules/.pnpm/preact@10.22.0/node_modules/preact'
          },
          {
            statSize: 8618,
            label:
              '@stylexjs+stylex@0.9.3/node_modules/@stylexjs/stylex/lib/es/stylex.mjs',
            filename:
              'node_modules/.pnpm/@stylexjs+stylex@0.9.3/node_modules/@stylexjs/stylex/lib/es/stylex.mjs'
          },
          {
            statSize: 797,
            label: 'foxact@0.2.35_react@18.3.1/node_modules/foxact',
            groups: [
              {
                statSize: 177,
                label: 'compose-context-provider/index.mjs',
                filename:
                  'node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/compose-context-provider/index.mjs'
              },
              {
                statSize: 33,
                label: 'noop/index.mjs',
                filename:
                  'node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/noop/index.mjs'
              },
              {
                statSize: 377,
                label: 'context-state/index.mjs',
                filename:
                  'node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/context-state/index.mjs'
              },
              {
                statSize: 210,
                label: 'use-abortable-effect/index.mjs',
                filename:
                  'node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact/use-abortable-effect/index.mjs'
              }
            ],
            filename:
              'node_modules/.pnpm/foxact@0.2.35_react@18.3.1/node_modules/foxact'
          },
          {
            statSize: 52123,
            label: 'squarified@0.3.2/node_modules/squarified/dist/index.mjs',
            filename:
              'node_modules/.pnpm/squarified@0.3.2/node_modules/squarified/dist/index.mjs'
          },
          {
            statSize: 388,
            label: 'clsx@2.1.1/node_modules/clsx/dist/clsx.mjs',
            filename:
              'node_modules/.pnpm/clsx@2.1.1/node_modules/clsx/dist/clsx.mjs'
          }
        ],
        filename: 'node_modules/.pnpm'
      }
    ],
    isAsset: true,
    isEntry: true,
    imports: []
  },
  {
    filename: 'index.html',
    label: 'index.html',
    parsedSize: 1776,
    mapSize: 0,
    statSize: 1776,
    gzipSize: 1133,
    brotliSize: 960,
    source: [
      {
        parsedSize: 1776,
        gzipSize: 1133,
        brotliSize: 960,
        label: 'index.html',
        filename: 'index.html'
      }
    ],
    stats: [
      {
        statSize: 1776,
        label: 'index.html',
        filename: 'index.html'
      }
    ],
    isAsset: true,
    isEntry: false,
    imports: []
  }
];

const client = new AnalyzerClient();

client.render('#app');

client.setOptions({
  sizes: 'statSize',
  analyzeModule: data
});
