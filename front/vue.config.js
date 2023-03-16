const { defineConfig } = require("@vue/cli-service");
module.exports = defineConfig({
  devServer: {
    headers: {         
    }
  },
  transpileDependencies: [
    'vue-meta',
  ],
  lintOnSave: false,
});
