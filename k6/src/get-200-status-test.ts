import { sleep, check } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

export let options: Options = {
  vus: 10000,
  duration: '100s'
};

export default () => {
  const res = http.get('http://localhost');
  check(res, {
    'status is 200': () => res.status === 200,
  });
  sleep(1);
};
