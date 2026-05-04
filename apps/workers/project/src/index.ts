export { Project } from './project/project.js';

export default {
  fetch() {
    return new Response('Project DO - Access via binding', { status: 404 });
  },
};
