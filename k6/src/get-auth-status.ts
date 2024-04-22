import { sleep, check } from 'k6';
import { Options } from 'k6/options';
import http from 'k6/http';

export let options: Options = {
  vus: 10000,
  duration: '10s',
};

export default () => {
  const res = http.get('http://localhost/auth/verify_session');
  check(res, {
    'status is 401': () => res.status === 401,
  });
  sleep(1);
};
