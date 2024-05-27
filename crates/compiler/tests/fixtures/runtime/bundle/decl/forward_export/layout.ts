function Layout() {}

function wrap(fn) {
  return () => {
    return fn()
  }
}

var ForwardLayout = wrap(Layout);
var LayoutComponent = ForwardLayout;

LayoutComponent.Sider = () => {};
LayoutComponent.Row = () => {}

export default LayoutComponent;