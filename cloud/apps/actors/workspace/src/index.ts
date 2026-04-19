export { Workspace } from './workspace/workspace.js';

export default {
  fetch() {
    return new Response('Workspace DO - Access via binding', { status: 404 });
  },
};
