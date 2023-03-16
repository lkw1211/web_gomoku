import axios from 'axios';

axios.defaults.baseURL = `${document.location.protocol}//${document.location.hostname}:8081`;

export default axios;