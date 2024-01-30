import {
  defineComponent as L,
  useSlots as $e,
  computed as d,
  openBlock as g,
  createElementBlock as $,
  normalizeClass as f,
  normalizeStyle as xe,
  createElementVNode as y,
  Fragment as O,
  withDirectives as B,
  unref as u,
  renderSlot as m,
  vShow as z,
  createCommentVNode as N,
  createStaticVNode as we,
  createVNode as Me,
  withModifiers as Be,
  createBlock as te,
  resolveDynamicComponent as ze,
  mergeProps as je,
  withCtx as Z
} from 'vue';

const de = '__SCROLL_EL_ID__',
  rt = de,
  be = 100;

function Ie(t) {
  return {
    '--soy-header-height': `${t.headerHeight}px`,
    '--soy-header-z-index': t.headerZIndex,
    '--soy-tab-height': `${t.tabHeight}px`,
    '--soy-tab-z-index': t.tabZIndex,
    '--soy-sider-width': `${t.siderWidth}px`,
    '--soy-sider-collapsed-width': `${t.siderCollapsedWidth}px`,
    '--soy-sider-z-index': t.siderZIndex,
    '--soy-mobile-sider-z-index': t.mobileSiderZIndex,
    '--soy-footer-height': `${t.footerHeight}px`,
    '--soy-footer-z-index': t.footerZIndex
  };
}

function Ve(t) {
  const {
      mode: e,
      isMobile: o,
      maxZIndex: a = be,
      headerHeight: s,
      tabHeight: r,
      siderWidth: l,
      siderCollapsedWidth: i,
      footerHeight: h
    } = t,
    c = a - 3,
    C = a - 5,
    I = e === 'vertical' || o ? a - 1 : a - 4,
    V = o ? a - 2 : 0,
    M = a - 5;
  return Ie({
    headerHeight: s,
    headerZIndex: c,
    tabHeight: r,
    tabZIndex: C,
    siderWidth: l,
    siderZIndex: I,
    mobileSiderZIndex: V,
    siderCollapsedWidth: i,
    footerHeight: h,
    footerZIndex: M
  });
}

const Le = /* @__PURE__ */ L({
    name: 'AdminLayout',
    __name: 'index',
    props: {
      mode: { default: 'vertical' },
      isMobile: { type: Boolean },
      scrollMode: { default: 'content' },
      scrollElId: { default: de },
      scrollElClass: {},
      scrollWrapperClass: {},
      commonClass: { default: 'transition-all-300' },
      fixedTop: { type: Boolean, default: !0 },
      maxZIndex: { default: be },
      headerVisible: { type: Boolean, default: !0 },
      headerClass: {},
      headerHeight: { default: 56 },
      tabVisible: { type: Boolean, default: !0 },
      tabClass: {},
      tabHeight: { default: 48 },
      siderVisible: { type: Boolean, default: !0 },
      siderClass: {},
      mobileSiderClass: {},
      siderCollapse: { type: Boolean, default: !1 },
      siderWidth: { default: 220 },
      siderCollapsedWidth: { default: 64 },
      contentClass: {},
      fullContent: { type: Boolean },
      footerVisible: { type: Boolean, default: !0 },
      fixedFooter: { type: Boolean },
      footerClass: {},
      footerHeight: { default: 48 },
      rightFooter: { type: Boolean, default: !1 }
    },
    emits: ['click-mobile-sider-mask'],
    setup(t, { emit: e }) {
      const o = t,
        a = $e(),
        s = d(() => Ve(o)),
        r = d(() => !!a.header && o.headerVisible),
        l = d(() => !!a.tab && o.tabVisible),
        i = d(() => !o.isMobile && !!a.sider && o.siderVisible),
        h = d(() => o.isMobile && !!a.sider && o.siderVisible),
        c = d(() => !!a.footer && o.footerVisible),
        C = d(() => o.scrollMode === 'wrapper'),
        I = d(() => o.scrollMode === 'content'),
        V = d(() => o.mode === 'vertical'),
        M = d(() => o.mode === 'horizontal'),
        j = d(() => o.fixedTop || (M.value && C.value)),
        T = d(() =>
          !o.fullContent && i.value
            ? o.siderCollapse
              ? p['left-gap_collapsed']
              : p['left-gap']
            : ''
        ),
        K = d(() => (V.value ? T.value : '')),
        Q = d(() => {
          const n = V.value,
            ee = M.value && C.value && !o.fixedFooter,
            ke = !!(M.value && o.rightFooter);
          return n || ee || ke ? T.value : '';
        }),
        _e = d(() => {
          let n = '';
          return (
            r.value && !K.value && (n += p['sider-padding-top']),
            c.value && !Q.value && (n += ` ${p['sider-padding-bottom']}`),
            n
          );
        });
      function Ce() {
        e('click-mobile-sider-mask');
      }
      return (n, ee) => (
        g(),
        $(
          'div',
          {
            class: f(['soybeanjs-qyp971', n.commonClass]),
            style: xe(s.value)
          },
          [
            y(
              'div',
              {
                id: C.value ? n.scrollElId : void 0,
                class: f([
                  'soybeanjs-jpgwa8',
                  n.commonClass,
                  n.scrollWrapperClass,
                  { 'soybeanjs-n12do3': C.value }
                ])
              },
              [
                r.value
                  ? (g(),
                    $(
                      O,
                      { key: 0 },
                      [
                        B(
                          y(
                            'header',
                            {
                              class: f([
                                u(p)['layout-header'],
                                'soybeanjs-gpr0x9',
                                n.commonClass,
                                n.headerClass,
                                K.value,
                                { 'soybeanjs-ihf5pz': j.value }
                              ])
                            },
                            [m(n.$slots, 'header')],
                            2
                          ),
                          [[z, !n.fullContent]]
                        ),
                        B(
                          y(
                            'div',
                            {
                              class: f([
                                u(p)['layout-header-placement'],
                                'soybeanjs-hg8qlw'
                              ])
                            },
                            null,
                            2
                          ),
                          [[z, !n.fullContent && j.value]]
                        )
                      ],
                      64
                    ))
                  : N('', !0),
                l.value
                  ? (g(),
                    $(
                      O,
                      { key: 1 },
                      [
                        B(
                          y(
                            'div',
                            {
                              class: f([
                                u(p)['layout-tab'],
                                'soybeanjs-gpr0x9',
                                n.commonClass,
                                n.tabClass,
                                { 'top-0!': !r.value },
                                T.value,
                                { 'soybeanjs-elvq0l': j.value }
                              ])
                            },
                            [m(n.$slots, 'tab')],
                            2
                          ),
                          [[z, !n.fullContent]]
                        ),
                        B(
                          y(
                            'div',
                            {
                              class: f([
                                u(p)['layout-tab-placement'],
                                'soybeanjs-hg8qlw'
                              ])
                            },
                            null,
                            2
                          ),
                          [[z, !n.fullContent && j.value]]
                        )
                      ],
                      64
                    ))
                  : N('', !0),
                i.value
                  ? B(
                      (g(),
                      $(
                        'aside',
                        {
                          key: 2,
                          class: f([
                            'soybeanjs-sbowzg',
                            n.commonClass,
                            n.siderClass,
                            _e.value,
                            n.siderCollapse
                              ? u(p)['layout-sider_collapsed']
                              : u(p)['layout-sider']
                          ])
                        },
                        [m(n.$slots, 'sider')],
                        2
                      )),
                      [[z, !n.fullContent]]
                    )
                  : N('', !0),
                h.value
                  ? (g(),
                    $(
                      O,
                      { key: 3 },
                      [
                        y(
                          'aside',
                          {
                            class: f([
                              'soybeanjs-lor397',
                              n.commonClass,
                              n.mobileSiderClass,
                              u(p)['layout-mobile-sider'],
                              n.siderCollapse
                                ? 'overflow-hidden'
                                : u(p)['layout-sider']
                            ])
                          },
                          [m(n.$slots, 'sider')],
                          2
                        ),
                        B(
                          y(
                            'div',
                            {
                              class: f([
                                'soybeanjs-4ibers',
                                u(p)['layout-mobile-sider-mask']
                              ]),
                              onClick: Ce
                            },
                            null,
                            2
                          ),
                          [[z, !n.siderCollapse]]
                        )
                      ],
                      64
                    ))
                  : N('', !0),
                y(
                  'main',
                  {
                    id: I.value ? n.scrollElId : void 0,
                    class: f([
                      'soybeanjs-fg4g4j',
                      n.commonClass,
                      n.contentClass,
                      T.value,
                      { 'soybeanjs-n12do3': I.value }
                    ])
                  },
                  [m(n.$slots, 'default')],
                  10,
                  He
                ),
                c.value
                  ? (g(),
                    $(
                      O,
                      { key: 4 },
                      [
                        B(
                          y(
                            'footer',
                            {
                              class: f([
                                u(p)['layout-footer'],
                                'soybeanjs-gpr0x9',
                                n.commonClass,
                                n.footerClass,
                                Q.value,
                                { 'soybeanjs-muaizb': n.fixedFooter }
                              ])
                            },
                            [m(n.$slots, 'footer')],
                            2
                          ),
                          [[z, !n.fullContent]]
                        ),
                        B(
                          y(
                            'div',
                            {
                              class: f([
                                u(p)['layout-footer-placement'],
                                'soybeanjs-hg8qlw'
                              ])
                            },
                            null,
                            2
                          ),
                          [[z, !n.fullContent && n.fixedFooter]]
                        )
                      ],
                      64
                    ))
                  : N('', !0)
              ],
              10,
              Ne
            )
          ],
          6
        )
      );
    }
  }),
  p = {
    'layout-header': '_layout-header_c343q_3',
    'layout-header-placement': '_layout-header-placement_c343q_4',
    'layout-tab': '_layout-tab_c343q_12',
    'layout-tab-placement': '_layout-tab-placement_c343q_18',
    'layout-sider': '_layout-sider_c343q_22',
    'layout-mobile-sider': '_layout-mobile-sider_c343q_27',
    'layout-mobile-sider-mask': '_layout-mobile-sider-mask_c343q_31',
    'layout-sider_collapsed': '_layout-sider_collapsed_c343q_35',
    'layout-footer': '_layout-footer_c343q_40',
    'layout-footer-placement': '_layout-footer-placement_c343q_41',
    'left-gap': '_left-gap_c343q_49',
    'left-gap_collapsed': '_left-gap_collapsed_c343q_53',
    'sider-padding-top': '_sider-padding-top_c343q_57',
    'sider-padding-bottom': '_sider-padding-bottom_c343q_61'
  },
  He = ['id'],
  Ne = ['id'];

export { Le as AdminLayout };
