import React from 'react';
import { useState } from 'react';
import { Button, Modal } from 'antd';
import {
  LaptopOutlined,
  NotificationOutlined,
  UserOutlined
} from '@ant-design/icons';
import type { MenuProps } from 'antd';
import { Breadcrumb, Layout, Menu, theme } from 'antd';
import { Button as LibButton } from '@farmfe-examples/lib-for-browser';
import { useNavigate, Outlet } from 'react-router-dom';
import { ProChat } from '@ant-design/pro-chat';

import './main.css';
import logo from '../assets/logo.png';
// import axios from 'axios';

const { Header, Content, Sider } = Layout;

// axios.get('https://music-erkelost.vercel.app/banner').then((res) => {
//   console.log(res);
// });

// axios
//   .get('/api')
//   .then((response) => {
//     console.log(response.data);
//   })
//   .catch((error) => {
//     console.error('There was an error!', error);
//   });

const items1: MenuProps['items'] = ['1', '2', '3'].map((key) => ({
  key,
  label: `nav ${key}`
}));

const items2: MenuProps['items'] = [
  UserOutlined,
  LaptopOutlined,
  NotificationOutlined
].map((icon, index) => {
  const key = String(index + 1);

  return {
    key: `sub${key}`,
    icon: React.createElement(icon),
    label: `subnav ${key}`,

    children: new Array(4).fill(null).map((_, j) => {
      const subKey = index * 4 + j + 1;
      return {
        key: subKey,
        label: `option${subKey}`
      };
    })
  };
});

export const AntdLayout: React.FC = () => {
  const {
    token: { colorBgContainer }
  } = theme.useToken();
  const navigate = useNavigate();
  const [isModalOpen, setIsModalOpen] = useState(false);

  const showModal = () => {
    setIsModalOpen(true);
  };

  const handleOk = () => {
    setIsModalOpen(false);
  };

  const handleCancel = () => {
    setIsModalOpen(false);
  };

  return (
    <Layout>
      <Header className="header">
        <div className="logo" />
        <Menu
          theme="dark"
          mode="horizontal"
          defaultSelectedKeys={['2']}
          items={items1}
          onClick={({ key }) => navigate(`/${key}`)}
        />
      </Header>
      <Layout>
        <Sider width={200} style={{ background: colorBgContainer }}>
          <Menu
            mode="inline"
            defaultSelectedKeys={['1']}
            defaultOpenKeys={['sub1']}
            style={{ height: '100%', borderRight: 0 }}
            items={items2}
          />
        </Sider>
        <Layout style={{ padding: '0 24px 24px' }}>
          <Breadcrumb style={{ margin: '16px 0' }}>
            <Breadcrumb.Item>Home</Breadcrumb.Item>
            <Breadcrumb.Item>List</Breadcrumb.Item>
            <Breadcrumb.Item>App</Breadcrumb.Item>
          </Breadcrumb>
          <Content
            style={{
              padding: 24,
              margin: 0,
              minHeight: 280,
              background: colorBgContainer
            }}
          >
            <ProChat />
            <div>
              <img width={600} src={logo} />
            </div>
            <LibButton /> 
            <Button type="primary" onClick={showModal}>
              Open Modal
            </Button>

            <Modal
              title="Basic Modal"
              open={isModalOpen}
              onOk={handleOk}
              onCancel={handleCancel}
            >
              <p>Some contents...</p>
              <p>Some contents...</p>
              <p>Some contents...</p>
            </Modal>
            <Outlet />
          </Content>
        </Layout>
      </Layout>
    </Layout>
  );
};
