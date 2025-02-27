const baseURL = '/__visualizer';

function request<T>(url: string, init?: RequestInit): Promise<T> {
  return fetch(baseURL + url, {
    method: 'GET',
    headers: {
      'Content-Type': 'application/json'
    },
    ...init
  })
    .then((res) => {
      if (!res.ok) {
        throw new Error(`HTTP error! status: ${res.status}`);
      }
      return res.json() as Promise<T>;
    })
    .catch((error) => {
      console.error('API Request failed:', error);
      throw error;
    });
}

// full see the server side
export const apis = {
  getResouce: () => request('/resource'),
  getModules: () => request('/modules'),
  getEnvInfo: () => request('/env_info'),
  getStats: () => request('/stats')
};
