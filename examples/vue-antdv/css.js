
const cssCode = `
#components-layout-demo-top-side-2 .logo {
  float: left;
  width: 120px;
  height: 31px;
  margin: 16px 24px 16px 0;
  background: rgba(255, 255, 255, 0.3);
}
.ant-row-rtl #components-layout-demo-top-side-2 .logo {
  float: right;
  margin: 16px 0 16px 24px;
}
.site-layout-background {
  background: #fff;
}
`;
const farmId = 'src/main.vue?lang=css&index=0&vue&t=0&hash=7c8a8451';
const previousStyle = document.querySelector(`style[data-farm-id="${farmId}"]`);
const style = document.createElement('style');
style.setAttribute('data-farm-id', farmId);
style.innerHTML = cssCode;
if (previousStyle) {
previousStyle.replaceWith(style);
} else {
document.head.appendChild(style);
}
module.meta.hot.accept();

module.onDispose(() => {
style.remove();
});
