import { generatePermission } from '@/routes';
import { isSSR } from '@/utils/is';
import setupMock from '@/utils/setupMock';
import Mock from 'mockjs';

if (!isSSR) {
  Mock.XHR.prototype.withCredentials = true;

  setupMock({
    setup: () => {
      // 用户信息
      const userRole = window.localStorage.getItem('userRole') || 'admin';
      Mock.mock(new RegExp('/api/user/userInfo'), () => {
        return Mock.mock({
          name: '王立群',
          avatar:
            'data:image/svg+xml;base64,PHN2ZyBmaWxsPSIjOWYxYThmIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PHBhdGggZD0iTTE5IDZWNUEyIDIgMCAwIDAgMTcgM0gxNUEyIDIgMCAwIDAgMTMgNVY2SDExVjVBMiAyIDAgMCAwIDkgM0g3QTIgMiAwIDAgMCA1IDVWNkgzVjIwSDEzLjA5QTUuNDcgNS40NyAwIDAgMSAxMyAxOUE2IDYgMCAwIDEgMjEgMTMuMzRWNk0yMCAxNVYxOEgyM1YyMEgyMFYyM0gxOFYyMEgxNVYxOEgxOFYxNVoiIC8+PC9zdmc+',
          email: 'wangliqun@email.com',
          job: 'frontend',
          jobName: '前端开发工程师',
          organization: 'Frontend',
          organizationName: '前端',
          location: 'beijing',
          locationName: '北京',
          introduction: '王力群并非是一个真实存在的人。',
          personalWebsite: 'https://www.arco.design',
          verified: true,
          phoneNumber: /177[*]{6}[0-9]{2}/,
          accountId: /[a-z]{4}[-][0-9]{8}/,
          registrationTime: Mock.Random.datetime('yyyy-MM-dd HH:mm:ss'),
          permissions: generatePermission(userRole)
        });
      });

      // 登录
      Mock.mock(new RegExp('/api/user/login'), (params) => {
        const { userName, password } = JSON.parse(params.body);
        if (!userName) {
          return {
            status: 'error',
            msg: '用户名不能为空'
          };
        }
        if (!password) {
          return {
            status: 'error',
            msg: '密码不能为空'
          };
        }
        if (userName === 'admin' && password === 'admin') {
          return {
            status: 'ok'
          };
        }
        return {
          status: 'error',
          msg: '账号或者密码错误'
        };
      });
    }
  });
}
