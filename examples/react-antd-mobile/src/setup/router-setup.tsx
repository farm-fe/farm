/// doc: https://reactrouter.com/en/v6.3.0/upgrading/v5#:%7E:text=single%20route%20config%3A-,//%20This%20is%20a%20React%20Router%20v6%20app,%7D,-This%20step%20is
// import { Router } from "react-router";
import {
  createHashRouter,
  RouterProvider,
  // BrowserRouter,
  // Routes,
  // Route,
  // Router,
  // useLocation,
  // useRoutes,
  // useNavigate
} from "react-router-dom";

export { ScrollRestoration } from "react-router-dom"; // 记住滚动位置
// export const ScrollToTop = () => { // 滚动到顶部
//   const { pathname } = useLocation();

//   useEffect(() => {
//     window.scrollTo(0, 0);
//   }, [pathname]);
//   return null;
// }

import routes from "../routes";

export const router = createHashRouter(routes);

router.subscribe((state) => {
  // 滚动到顶部
  window.scrollTo(0, 0);
});

export default () => {
  // const history = createBrowserHistory();
  // router = useNavigate();
  return <RouterProvider router={router} />;
};
