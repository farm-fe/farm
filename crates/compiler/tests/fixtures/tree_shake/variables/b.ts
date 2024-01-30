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
const p = {
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
  Ne = ['id'],
  He = ['id'],
  Le = /* @__PURE__ */ L({
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
  });
var qe = { grad: 0.9, turn: 360, rad: 360 / (2 * Math.PI) },
  w = function (t) {
    return typeof t == 'string' ? t.length > 0 : typeof t == 'number';
  },
  b = function (t, e, o) {
    return (
      e === void 0 && (e = 0),
      o === void 0 && (o = Math.pow(10, e)),
      Math.round(o * t) / o + 0
    );
  },
  _ = function (t, e, o) {
    return (
      e === void 0 && (e = 0),
      o === void 0 && (o = 1),
      t > o ? o : t > e ? t : e
    );
  },
  he = function (t) {
    return (t = isFinite(t) ? t % 360 : 0) > 0 ? t : t + 360;
  },
  oe = function (t) {
    return {
      r: _(t.r, 0, 255),
      g: _(t.g, 0, 255),
      b: _(t.b, 0, 255),
      a: _(t.a)
    };
  },
  P = function (t) {
    return { r: b(t.r), g: b(t.g), b: b(t.b), a: b(t.a, 3) };
  },
  Se = /^#([0-9a-f]{3,8})$/i,
  R = function (t) {
    var e = t.toString(16);
    return e.length < 2 ? '0' + e : e;
  },
  fe = function (t) {
    var e = t.r,
      o = t.g,
      a = t.b,
      s = t.a,
      r = Math.max(e, o, a),
      l = r - Math.min(e, o, a),
      i = l
        ? r === e
          ? (o - a) / l
          : r === o
          ? 2 + (a - e) / l
          : 4 + (e - o) / l
        : 0;
    return {
      h: 60 * (i < 0 ? i + 6 : i),
      s: r ? (l / r) * 100 : 0,
      v: (r / 255) * 100,
      a: s
    };
  },
  pe = function (t) {
    var e = t.h,
      o = t.s,
      a = t.v,
      s = t.a;
    (e = (e / 360) * 6), (o /= 100), (a /= 100);
    var r = Math.floor(e),
      l = a * (1 - o),
      i = a * (1 - (e - r) * o),
      h = a * (1 - (1 - e + r) * o),
      c = r % 6;
    return {
      r: 255 * [a, i, l, l, h, a][c],
      g: 255 * [h, a, a, i, l, l][c],
      b: 255 * [l, l, h, a, a, i][c],
      a: s
    };
  },
  ae = function (t) {
    return { h: he(t.h), s: _(t.s, 0, 100), l: _(t.l, 0, 100), a: _(t.a) };
  },
  se = function (t) {
    return { h: b(t.h), s: b(t.s), l: b(t.l), a: b(t.a, 3) };
  },
  re = function (t) {
    return pe(
      ((o = (e = t).s),
      {
        h: e.h,
        s:
          (o *= ((a = e.l) < 50 ? a : 100 - a) / 100) > 0
            ? ((2 * o) / (a + o)) * 100
            : 0,
        v: a + o,
        a: e.a
      })
    );
    var e, o, a;
  },
  q = function (t) {
    return {
      h: (e = fe(t)).h,
      s:
        (s = ((200 - (o = e.s)) * (a = e.v)) / 100) > 0 && s < 200
          ? ((o * a) / 100 / (s <= 100 ? s : 200 - s)) * 100
          : 0,
      l: s / 2,
      a: e.a
    };
    var e, o, a, s;
  },
  Te =
    /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s*,\s*([+-]?\d*\.?\d+)%\s*,\s*([+-]?\d*\.?\d+)%\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i,
  Oe =
    /^hsla?\(\s*([+-]?\d*\.?\d+)(deg|rad|grad|turn)?\s+([+-]?\d*\.?\d+)%\s+([+-]?\d*\.?\d+)%\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i,
  Re =
    /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*,\s*([+-]?\d*\.?\d+)(%)?\s*(?:,\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i,
  Ee =
    /^rgba?\(\s*([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s+([+-]?\d*\.?\d+)(%)?\s*(?:\/\s*([+-]?\d*\.?\d+)(%)?\s*)?\)$/i,
  G = {
    string: [
      [
        function (t) {
          var e = Se.exec(t);
          return e
            ? (t = e[1]).length <= 4
              ? {
                  r: parseInt(t[0] + t[0], 16),
                  g: parseInt(t[1] + t[1], 16),
                  b: parseInt(t[2] + t[2], 16),
                  a: t.length === 4 ? b(parseInt(t[3] + t[3], 16) / 255, 2) : 1
                }
              : t.length === 6 || t.length === 8
              ? {
                  r: parseInt(t.substr(0, 2), 16),
                  g: parseInt(t.substr(2, 2), 16),
                  b: parseInt(t.substr(4, 2), 16),
                  a:
                    t.length === 8
                      ? b(parseInt(t.substr(6, 2), 16) / 255, 2)
                      : 1
                }
              : null
            : null;
        },
        'hex'
      ],
      [
        function (t) {
          var e = Re.exec(t) || Ee.exec(t);
          return e
            ? e[2] !== e[4] || e[4] !== e[6]
              ? null
              : oe({
                  r: Number(e[1]) / (e[2] ? 100 / 255 : 1),
                  g: Number(e[3]) / (e[4] ? 100 / 255 : 1),
                  b: Number(e[5]) / (e[6] ? 100 / 255 : 1),
                  a: e[7] === void 0 ? 1 : Number(e[7]) / (e[8] ? 100 : 1)
                })
            : null;
        },
        'rgb'
      ],
      [
        function (t) {
          var e = Te.exec(t) || Oe.exec(t);
          if (!e) return null;
          var o,
            a,
            s = ae({
              h:
                ((o = e[1]),
                (a = e[2]),
                a === void 0 && (a = 'deg'),
                Number(o) * (qe[a] || 1)),
              s: Number(e[3]),
              l: Number(e[4]),
              a: e[5] === void 0 ? 1 : Number(e[5]) / (e[6] ? 100 : 1)
            });
          return re(s);
        },
        'hsl'
      ]
    ],
    object: [
      [
        function (t) {
          var e = t.r,
            o = t.g,
            a = t.b,
            s = t.a,
            r = s === void 0 ? 1 : s;
          return w(e) && w(o) && w(a)
            ? oe({ r: Number(e), g: Number(o), b: Number(a), a: Number(r) })
            : null;
        },
        'rgb'
      ],
      [
        function (t) {
          var e = t.h,
            o = t.s,
            a = t.l,
            s = t.a,
            r = s === void 0 ? 1 : s;
          if (!w(e) || !w(o) || !w(a)) return null;
          var l = ae({
            h: Number(e),
            s: Number(o),
            l: Number(a),
            a: Number(r)
          });
          return re(l);
        },
        'hsl'
      ],
      [
        function (t) {
          var e = t.h,
            o = t.s,
            a = t.v,
            s = t.a,
            r = s === void 0 ? 1 : s;
          if (!w(e) || !w(o) || !w(a)) return null;
          var l = (function (i) {
            return {
              h: he(i.h),
              s: _(i.s, 0, 100),
              v: _(i.v, 0, 100),
              a: _(i.a)
            };
          })({ h: Number(e), s: Number(o), v: Number(a), a: Number(r) });
          return pe(l);
        },
        'hsv'
      ]
    ]
  },
  ne = function (t, e) {
    for (var o = 0; o < e.length; o++) {
      var a = e[o][0](t);
      if (a) return [a, e[o][1]];
    }
    return [null, void 0];
  },
  Ze = function (t) {
    return typeof t == 'string'
      ? ne(t.trim(), G.string)
      : typeof t == 'object' && t !== null
      ? ne(t, G.object)
      : [null, void 0];
  },
  A = function (t, e) {
    var o = q(t);
    return { h: o.h, s: _(o.s + 100 * e, 0, 100), l: o.l, a: o.a };
  },
  W = function (t) {
    return (299 * t.r + 587 * t.g + 114 * t.b) / 1e3 / 255;
  },
  le = function (t, e) {
    var o = q(t);
    return { h: o.h, s: o.s, l: _(o.l + 100 * e, 0, 100), a: o.a };
  },
  U = (function () {
    function t(e) {
      (this.parsed = Ze(e)[0]),
        (this.rgba = this.parsed || { r: 0, g: 0, b: 0, a: 1 });
    }
    return (
      (t.prototype.isValid = function () {
        return this.parsed !== null;
      }),
      (t.prototype.brightness = function () {
        return b(W(this.rgba), 2);
      }),
      (t.prototype.isDark = function () {
        return W(this.rgba) < 0.5;
      }),
      (t.prototype.isLight = function () {
        return W(this.rgba) >= 0.5;
      }),
      (t.prototype.toHex = function () {
        return (
          (e = P(this.rgba)),
          (o = e.r),
          (a = e.g),
          (s = e.b),
          (l = (r = e.a) < 1 ? R(b(255 * r)) : ''),
          '#' + R(o) + R(a) + R(s) + l
        );
        var e, o, a, s, r, l;
      }),
      (t.prototype.toRgb = function () {
        return P(this.rgba);
      }),
      (t.prototype.toRgbString = function () {
        return (
          (e = P(this.rgba)),
          (o = e.r),
          (a = e.g),
          (s = e.b),
          (r = e.a) < 1
            ? 'rgba(' + o + ', ' + a + ', ' + s + ', ' + r + ')'
            : 'rgb(' + o + ', ' + a + ', ' + s + ')'
        );
        var e, o, a, s, r;
      }),
      (t.prototype.toHsl = function () {
        return se(q(this.rgba));
      }),
      (t.prototype.toHslString = function () {
        return (
          (e = se(q(this.rgba))),
          (o = e.h),
          (a = e.s),
          (s = e.l),
          (r = e.a) < 1
            ? 'hsla(' + o + ', ' + a + '%, ' + s + '%, ' + r + ')'
            : 'hsl(' + o + ', ' + a + '%, ' + s + '%)'
        );
        var e, o, a, s, r;
      }),
      (t.prototype.toHsv = function () {
        return (
          (e = fe(this.rgba)), { h: b(e.h), s: b(e.s), v: b(e.v), a: b(e.a, 3) }
        );
        var e;
      }),
      (t.prototype.invert = function () {
        return v({
          r: 255 - (e = this.rgba).r,
          g: 255 - e.g,
          b: 255 - e.b,
          a: e.a
        });
        var e;
      }),
      (t.prototype.saturate = function (e) {
        return e === void 0 && (e = 0.1), v(A(this.rgba, e));
      }),
      (t.prototype.desaturate = function (e) {
        return e === void 0 && (e = 0.1), v(A(this.rgba, -e));
      }),
      (t.prototype.grayscale = function () {
        return v(A(this.rgba, -1));
      }),
      (t.prototype.lighten = function (e) {
        return e === void 0 && (e = 0.1), v(le(this.rgba, e));
      }),
      (t.prototype.darken = function (e) {
        return e === void 0 && (e = 0.1), v(le(this.rgba, -e));
      }),
      (t.prototype.rotate = function (e) {
        return e === void 0 && (e = 15), this.hue(this.hue() + e);
      }),
      (t.prototype.alpha = function (e) {
        return typeof e == 'number'
          ? v({ r: (o = this.rgba).r, g: o.g, b: o.b, a: e })
          : b(this.rgba.a, 3);
        var o;
      }),
      (t.prototype.hue = function (e) {
        var o = q(this.rgba);
        return typeof e == 'number'
          ? v({ h: e, s: o.s, l: o.l, a: o.a })
          : b(o.h);
      }),
      (t.prototype.isEqual = function (e) {
        return this.toHex() === v(e).toHex();
      }),
      t
    );
  })(),
  v = function (t) {
    return t instanceof U ? t : new U(t);
  },
  ie = [],
  Pe = function (t) {
    t.forEach(function (e) {
      ie.indexOf(e) < 0 && (e(U, G), ie.push(e));
    });
  },
  x = function (t, e, o) {
    return (
      e === void 0 && (e = 0),
      o === void 0 && (o = 1),
      t > o ? o : t > e ? t : e
    );
  },
  F = function (t) {
    var e = t / 255;
    return e < 0.04045 ? e / 12.92 : Math.pow((e + 0.055) / 1.055, 2.4);
  },
  D = function (t) {
    return (
      255 * (t > 31308e-7 ? 1.055 * Math.pow(t, 1 / 2.4) - 0.055 : 12.92 * t)
    );
  },
  X = 96.422,
  Y = 100,
  J = 82.521,
  Ae = function (t) {
    var e,
      o,
      a = {
        x: 0.9555766 * (e = t).x + -0.0230393 * e.y + 0.0631636 * e.z,
        y: -0.0282895 * e.x + 1.0099416 * e.y + 0.0210077 * e.z,
        z: 0.0122982 * e.x + -0.020483 * e.y + 1.3299098 * e.z
      };
    return (
      (o = {
        r: D(0.032404542 * a.x - 0.015371385 * a.y - 4985314e-9 * a.z),
        g: D(-969266e-8 * a.x + 0.018760108 * a.y + 41556e-8 * a.z),
        b: D(556434e-9 * a.x - 2040259e-9 * a.y + 0.010572252 * a.z),
        a: t.a
      }),
      { r: x(o.r, 0, 255), g: x(o.g, 0, 255), b: x(o.b, 0, 255), a: x(o.a) }
    );
  },
  We = function (t) {
    var e = F(t.r),
      o = F(t.g),
      a = F(t.b);
    return (function (s) {
      return { x: x(s.x, 0, X), y: x(s.y, 0, Y), z: x(s.z, 0, J), a: x(s.a) };
    })(
      (function (s) {
        return {
          x: 1.0478112 * s.x + 0.0228866 * s.y + -0.050127 * s.z,
          y: 0.0295424 * s.x + 0.9904844 * s.y + -0.0170491 * s.z,
          z: -92345e-7 * s.x + 0.0150436 * s.y + 0.7521316 * s.z,
          a: s.a
        };
      })({
        x: 100 * (0.4124564 * e + 0.3575761 * o + 0.1804375 * a),
        y: 100 * (0.2126729 * e + 0.7151522 * o + 0.072175 * a),
        z: 100 * (0.0193339 * e + 0.119192 * o + 0.9503041 * a),
        a: t.a
      })
    );
  },
  S = 216 / 24389,
  H = 24389 / 27,
  ue = function (t) {
    var e = We(t),
      o = e.x / X,
      a = e.y / Y,
      s = e.z / J;
    return (
      (o = o > S ? Math.cbrt(o) : (H * o + 16) / 116),
      {
        l: 116 * (a = a > S ? Math.cbrt(a) : (H * a + 16) / 116) - 16,
        a: 500 * (o - a),
        b: 200 * (a - (s = s > S ? Math.cbrt(s) : (H * s + 16) / 116)),
        alpha: e.a
      }
    );
  },
  Fe = function (t, e, o) {
    var a,
      s = ue(t),
      r = ue(e);
    return (function (l) {
      var i = (l.l + 16) / 116,
        h = l.a / 500 + i,
        c = i - l.b / 200;
      return Ae({
        x: (Math.pow(h, 3) > S ? Math.pow(h, 3) : (116 * h - 16) / H) * X,
        y: (l.l > 8 ? Math.pow((l.l + 16) / 116, 3) : l.l / H) * Y,
        z: (Math.pow(c, 3) > S ? Math.pow(c, 3) : (116 * c - 16) / H) * J,
        a: l.alpha
      });
    })({
      l: x(
        (a = {
          l: s.l * (1 - o) + r.l * o,
          a: s.a * (1 - o) + r.a * o,
          b: s.b * (1 - o) + r.b * o,
          alpha: s.alpha * (1 - o) + r.alpha * o
        }).l,
        0,
        400
      ),
      a: a.a,
      b: a.b,
      alpha: x(a.alpha)
    });
  };
function De(t) {
  function e(o, a, s) {
    s === void 0 && (s = 5);
    for (var r = [], l = 1 / (s - 1), i = 0; i <= s - 1; i++)
      r.push(o.mix(a, l * i));
    return r;
  }
  (t.prototype.mix = function (o, a) {
    a === void 0 && (a = 0.5);
    var s = o instanceof t ? o : new t(o),
      r = Fe(this.toRgb(), s.toRgb(), a);
    return new t(r);
  }),
    (t.prototype.tints = function (o) {
      return e(this, '#fff', o);
    }),
    (t.prototype.shades = function (o) {
      return e(this, '#000', o);
    }),
    (t.prototype.tones = function (o) {
      return e(this, '#808080', o);
    });
}
Pe([De]);
function me(t, e) {
  t.component(e.name, e);
}
function E(t, e) {
  return v(t).alpha(e).toHex();
}
function ce(t, e, o = '#ffffff') {
  const a = E(t, e),
    { r: s, g: r, b: l } = v(a).toRgb(),
    { r: i, g: h, b: c } = v(o).toRgb();
  function C(V, M, j) {
    return M + (V - M) * j;
  }
  const I = {
    r: C(s, i, e),
    g: C(r, h, e),
    b: C(l, c, e)
  };
  return v(I).toHex();
}
Le.install = me;
const Ge = '#1890ff';
function Ue(t) {
  return {
    '--soy-primary-color': t.primaryColor,
    '--soy-primary-color1': t.primaryColor1,
    '--soy-primary-color2': t.primaryColor2,
    '--soy-primary-color-opacity1': t.primaryColorOpacity1,
    '--soy-primary-color-opacity2': t.primaryColorOpacity2,
    '--soy-primary-color-opacity3': t.primaryColorOpacity3
  };
}
function Xe(t) {
  const e = {
    primaryColor: t,
    primaryColor1: ce(t, 0.1, '#ffffff'),
    primaryColor2: ce(t, 0.3, '#000000'),
    primaryColorOpacity1: E(t, 0.1),
    primaryColorOpacity2: E(t, 0.15),
    primaryColorOpacity3: E(t, 0.3)
  };
  return Ue(e);
}
const Ye = { style: { width: '100%', height: '100%' } },
  Je = /* @__PURE__ */ we(
    '<defs><symbol id="geometry-left" viewBox="0 0 214 36"><path d="M17 0h197v36H0v-2c4.5 0 9-3.5 9-8V8c0-4.5 3.5-8 8-8z"></path></symbol><symbol id="geometry-right" viewBox="0 0 214 36"><use xlink:href="#geometry-left"></use></symbol><clipPath><rect width="100%" height="100%" x="0"></rect></clipPath></defs><svg width="51%" height="100%"><use xlink:href="#geometry-left" width="214" height="36" fill="currentColor"></use></svg><g transform="scale(-1, 1)"><svg width="51%" height="100%" x="-100%" y="0"><use xlink:href="#geometry-right" width="214" height="36" fill="currentColor"></use></svg></g>',
    3
  ),
  Ke = [Je],
  Qe = /* @__PURE__ */ L({
    name: 'ChromeTabBg',
    __name: 'chrome-tab-bg',
    setup(t) {
      return (e, o) => (g(), $('svg', Ye, Ke));
    }
  }),
  k = {
    'button-tab': '_button-tab_15sm7_3',
    'button-tab_dark': '_button-tab_dark_15sm7_7',
    'button-tab_active': '_button-tab_active_15sm7_16',
    'button-tab_active_dark': '_button-tab_active_dark_15sm7_22',
    'icon-close': '_icon-close_15sm7_26',
    'chrome-tab': '_chrome-tab_15sm7_36',
    'chrome-tab_active': '_chrome-tab_active_15sm7_40',
    'chrome-tab__bg': '_chrome-tab__bg_15sm7_45',
    'chrome-tab_active_dark': '_chrome-tab_active_dark_15sm7_53',
    'chrome-tab_dark': '_chrome-tab_dark_15sm7_65',
    'chrome-tab-divider': '_chrome-tab-divider_15sm7_87'
  },
  ye = /* @__PURE__ */ L({
    name: 'ChromeTab',
    __name: 'chrome-tab',
    props: {
      darkMode: { type: Boolean },
      mode: {},
      commonClass: {},
      buttonClass: {},
      chromeClass: {},
      active: { type: Boolean },
      activeColor: {},
      closable: { type: Boolean }
    },
    setup(t) {
      return (e, o) => (
        g(),
        $(
          'div',
          {
            class: f([
              'soybeanjs-yxkfns',
              u(k)['chrome-tab'],
              { [u(k)['chrome-tab_dark']]: e.darkMode },
              { [u(k)['chrome-tab_active']]: e.active },
              { [u(k)['chrome-tab_active_dark']]: e.active && e.darkMode }
            ])
          },
          [
            y(
              'div',
              {
                class: f(['soybeanjs-pr5008', u(k)['chrome-tab__bg']])
              },
              [Me(Qe)],
              2
            ),
            m(e.$slots, 'prefix'),
            m(e.$slots, 'default'),
            m(e.$slots, 'suffix'),
            y(
              'div',
              {
                class: f(['soybeanjs-714u3q', u(k)['chrome-tab-divider']])
              },
              null,
              2
            )
          ],
          2
        )
      );
    }
  }),
  ve = /* @__PURE__ */ L({
    name: 'ButtonTab',
    __name: 'button-tab',
    props: {
      darkMode: { type: Boolean },
      mode: {},
      commonClass: {},
      buttonClass: {},
      chromeClass: {},
      active: { type: Boolean },
      activeColor: {},
      closable: { type: Boolean }
    },
    setup(t) {
      return (e, o) => (
        g(),
        $(
          'div',
          {
            class: f([
              'soybeanjs-x463fz',
              u(k)['button-tab'],
              { [u(k)['button-tab_dark']]: e.darkMode },
              { [u(k)['button-tab_active']]: e.active },
              { [u(k)['button-tab_active_dark']]: e.active && e.darkMode }
            ])
          },
          [
            m(e.$slots, 'prefix'),
            m(e.$slots, 'default'),
            m(e.$slots, 'suffix')
          ],
          2
        )
      );
    }
  }),
  et = ['onClick'],
  tt = /* @__PURE__ */ y(
    'svg',
    {
      width: '1em',
      height: '1em',
      viewBox: '0 0 1024 1024'
    },
    [
      /* @__PURE__ */ y('path', {
        fill: 'currentColor',
        d: 'm563.8 512l262.5-312.9c4.4-5.2.7-13.1-6.1-13.1h-79.8c-4.7 0-9.2 2.1-12.3 5.7L511.6 449.8L295.1 191.7c-3-3.6-7.5-5.7-12.3-5.7H203c-6.8 0-10.5 7.9-6.1 13.1L459.4 512L196.9 824.9A7.95 7.95 0 0 0 203 838h79.8c4.7 0 9.2-2.1 12.3-5.7l216.5-258.1l216.5 258.1c3 3.6 7.5 5.7 12.3 5.7h79.8c6.8 0 10.5-7.9 6.1-13.1L563.8 512z'
      })
    ],
    -1
  ),
  ot = [tt],
  at = /* @__PURE__ */ L({
    name: 'IconClose',
    __name: 'icon-close',
    emits: ['click'],
    setup(t, { emit: e }) {
      function o() {
        e('click');
      }
      return (a, s) => (
        g(),
        $(
          'div',
          {
            class: 'soybeanjs-bj4ztj',
            onClick: Be(o, ['stop'])
          },
          ot,
          8,
          et
        )
      );
    }
  }),
  ge = /* @__PURE__ */ L({
    name: 'PageTab',
    __name: 'index',
    props: {
      darkMode: { type: Boolean },
      mode: { default: 'chrome' },
      commonClass: { default: 'transition-all-300' },
      buttonClass: {},
      chromeClass: {},
      active: { type: Boolean },
      activeColor: { default: Ge },
      closable: { type: Boolean, default: !0 }
    },
    emits: ['close'],
    setup(t, { emit: e }) {
      const o = t,
        a = d(() => {
          const { mode: i, chromeClass: h, buttonClass: c } = o;
          return {
            chrome: {
              component: ye,
              class: h
            },
            button: {
              component: ve,
              class: c
            }
          }[i];
        }),
        s = d(() => Xe(o.activeColor)),
        r = d(() => {
          const { chromeClass: i, buttonClass: h, ...c } = o;
          return c;
        });
      function l() {
        e('close');
      }
      return (i, h) => (
        g(),
        te(
          ze(a.value.component),
          je(
            {
              class: a.value.class,
              style: s.value
            },
            r.value
          ),
          {
            prefix: Z(() => [m(i.$slots, 'prefix')]),
            suffix: Z(() => [
              m(i.$slots, 'suffix', {}, () => [
                i.closable
                  ? (g(),
                    te(
                      at,
                      {
                        key: 0,
                        class: f([u(k)['icon-close']]),
                        onClick: l
                      },
                      null,
                      8,
                      ['class']
                    ))
                  : N('', !0)
              ])
            ]),
            default: Z(() => [m(i.$slots, 'default')]),
            _: 3
          },
          16,
          ['class', 'style']
        )
      );
    }
  });
ge.install = me;
const nt = ge,
  lt = ve,
  it = ye;
export {
  Le as AdminLayout,
  nt as AdminTab,
  lt as ButtonTab,
  it as ChromeTab,
  be as LAYOUT_MAX_Z_INDEX,
  de as LAYOUT_SCROLL_EL_ID,
  ge as PageTab,
  rt as SCROLL_EL_ID
};
