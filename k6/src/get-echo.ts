import { check } from 'k6';
import http from 'k6/http';

export const options = {
  scenarios: {
    constant_request_rate: {
      executor: 'constant-arrival-rate',
      rate: 3000,
      timeUnit: '1s',
      duration: '30s',
      preAllocatedVUs: 3000,
      maxVUs: 10000,
    },
  },
};

export default () => {
  const res = http.get(`http://209.151.149.253/echo`);
  check(res, {
    'status is 200': r => r.status === 200,
  });
};
