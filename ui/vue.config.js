const { defineConfig } = require("@vue/cli-service");
module.exports = defineConfig({
  devServer: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
      "Access-Control-Allow-Origin": "*"
    }
  },
  transpileDependencies: [
    'vue-meta',
  ],
  lintOnSave: false,
  runtimeCompiler: true,
});