import { createRootRoute, HeadContent, Link, Outlet, Scripts } from '@tanstack/react-router'
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { Button, Result, Space } from "antd";
import { PageLoading } from "@ant-design/pro-components";

export const Route = createRootRoute({
  component: RootLayout,
  notFoundComponent: Root404,
  pendingComponent: RootPendingComponent,
  errorComponent: RootErrorComponent,
})

function RootLayout() {
  return <>
    <HeadContent />
    <Outlet />
    <Scripts />
    {/* @ts-ignore */}
    {process.env.NODE_ENV === 'development' && <TanStackRouterDevtools />}
  </>
}

function Root404() {
  return <Result
    status="404"
    title="404"
    subTitle="页面不存在"
    extra={<Link to={'/'}><Button type="primary">返回首页</Button></Link>}
  />
}

function RootPendingComponent() {
  return <PageLoading size="large" tip={'页面正在加载中，请稍候。。。'} delay={200}>
    <span />
  </PageLoading>
}

function RootErrorComponent(props: { error: any; reset: Function } & any) {
  const { error, reset } = props;
  console.log('RootErrorComponent', props)
  return <Result
    status="error"
    title={error.name}
    subTitle={<>
      <span>{error.message}</span>
      <pre style={{
        textAlign: "left",
        fontSize: '1em',
        border: '1px solid red',
        borderRadius: '.25rem',
        padding: '.3rem',
        color: 'red',
        overflow: 'auto',
      }}><code>{error.stack}</code></pre>
    </>}
    extra={<Space>
      <Link to={'/'}><Button type="primary">返回首页</Button></Link>
      <Button type="default" onClick={reset}>重置</Button>
    </Space>}
  />
}
