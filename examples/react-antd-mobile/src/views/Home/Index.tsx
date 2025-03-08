import { NavBar, Space, Swiper } from "antd-mobile";

const colors = ["#ace0ff", "#bcffbd", "#e4fabd", "#ffcfac"];

const items = colors.map((color, index) => (
  <Swiper.Item key={index}>
    <div style={{ background: color, height: "150px" }}>{index + 1}</div>
  </Swiper.Item>
));

export default () => {
  return (
    <Space direction="vertical" block>
      <NavBar back={null}>测试</NavBar>
      <Swiper autoplay loop>
        {items}
      </Swiper>
    </Space>
  );
};
