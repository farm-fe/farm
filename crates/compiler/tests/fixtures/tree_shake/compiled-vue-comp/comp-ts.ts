import { defineComponent as _defineComponent } from 'vue'
import { createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, withCtx as _withCtx, createVNode as _createVNode, createElementVNode as _createElementVNode, createStaticVNode as _createStaticVNode, Fragment as _Fragment, openBlock as _openBlock, createElementBlock as _createElementBlock, pushScopeId as _pushScopeId, popScopeId as _popScopeId } from "vue"
const _imports_0 = "URL_ADDRESS"


const _withScopeId = n => (_pushScopeId("data-v-370e85d7"),n=n(),_popScopeId(),n)
const _hoisted_1 = { class: "container" }
const _hoisted_2 = /*#__PURE__*/_createStaticVNode("<a href=\"https://farmfe.org/\" target=\"_blank\" data-v-370e85d7><div class=\"logo1\" data-v-370e85d7></div><div class=\"logo2\" data-v-370e85d7></div></a><a href=\"https://farmfe.org/\" target=\"_blank\" data-v-370e85d7><img src=\"" + _imports_0 + "\" class=\"logo\" alt=\"Farm logo\" data-v-370e85d7></a>", 2)
const _hoisted_4 = {
  href: "https://vuejs.org/",
  target: "_blank"
}

const HelloWorld = /*#__PURE__*/_defineComponent({});
const Formatter= /*#__PURE__*/_defineComponent({});


export default /*#__PURE__*/_defineComponent({
  __name: 'index',
  setup(__props) {


return (_ctx: any,_cache: any) => {
  const _component_el_button = _resolveComponent("el-button")!
  const _component_my_svg_icon = _resolveComponent("my-svg-icon")!
  const _component_el_config_provider = _resolveComponent("el-config-provider")!

  return (_openBlock(), _createElementBlock(_Fragment, null, [
    _createVNode(_component_el_button, {
      type: "primary",
      onClick: _cache[0] || (_cache[0] = ($event: any) => (_ctx.$router.push('/about')))
    }, {
      default: _withCtx(() => [
        _createTextVNode("to about page")
      ]),
      _: 1
    }),
    _createElementVNode("div", _hoisted_1, [
      _hoisted_2,
      _createElementVNode("a", _hoisted_4, [
        _createVNode(_component_my_svg_icon, {
          name: "icon-vue",
          class: "logo",
          style: {"height":"6.25rem","width":"6.25rem"}
        })
      ])
    ]),
    _createVNode(_component_el_config_provider, {
      size: 'large',
      "z-index": 3000
    }, {
      default: _withCtx(() => [
        _createVNode(HelloWorld, { msg: "Farm + Vue" }),
        _createVNode(Formatter)
      ]),
      _: 1
    })
  ], 64))
}
}

})