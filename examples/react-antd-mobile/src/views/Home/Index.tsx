import { NavBar, Space, Swiper } from "antd-mobile";
import { useEffect } from "react";

const colors = ["#ace0ff", "#bcffbd", "#e4fabd", "#ffcfac"];

const items = colors.map((color, index) => (
  <Swiper.Item key={index}>
    <div style={{ background: color, height: "150px" }}>{index + 1}</div>
  </Swiper.Item>
));

export default function Index() {
  async function asyncFunc() {
    await new Promise((resolve) =>
      setTimeout(() => {
        console.log("request 1");
        resolve(null);
      }, 0),
    );

    await new Promise((resolve) =>
      setTimeout(() => {
        console.log("request 2");
        resolve(null);
      }, 0),
    );

    console.log("request done");
  }
  useEffect(() => {
    console.log("useEffect");
    asyncFunc();
  }, []);

  return (
    <Space direction="vertical" block>
      <NavBar back={null}>测试</NavBar>
      <Swiper autoplay loop>
        {items}
      </Swiper>
    </Space>
  );
}
