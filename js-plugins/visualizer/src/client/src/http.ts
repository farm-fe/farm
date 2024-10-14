import type { AxiosError, AxiosRequestConfig } from 'axios';
import axios from 'axios';

type HttpConfig = AxiosRequestConfig & { skipInterceptors?: boolean };

export interface QueryParams {
  [key: string]:
    | undefined
    | number
    | boolean
    | string
    | Array<number | boolean | string>;
}

const axiosInstance = axios.create({
  timeout: 60000
});

axiosInstance.interceptors.response.use(
  (value) => {
    return value.data;
  },
  (error: AxiosError) => {
    return Promise.reject(error);
  }
);

export const http = {
  get<T = any>(
    url: string,
    params: QueryParams = {},
    config?: HttpConfig
  ): Promise<T> {
    return axiosInstance.get(url, {
      params,
      ...config
    });
  },
  post<T = any>(url: string, data: any, config?: HttpConfig): Promise<T> {
    return axiosInstance.post(url, data, config);
  },
  put<T = any>(url: string, data: any, config?: HttpConfig): Promise<T> {
    return axiosInstance.put(url, data, config);
  },
  delete<T = any>(
    url: string,
    params: QueryParams = {},
    config?: HttpConfig
  ): Promise<T> {
    return axiosInstance.delete(url, {
      params,
      ...config
    });
  }
};
