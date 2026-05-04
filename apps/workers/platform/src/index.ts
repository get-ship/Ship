export { Platform } from './platform/platform.js';

export default {
  fetch() {
    return new Response('Platform DO - Access via binding', { status: 404 });
  },
};
