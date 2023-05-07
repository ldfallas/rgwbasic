const CopyWebpackPlugin = require("copy-webpack-plugin");
const path = require('path');

module.exports = {
   entry: { bootstrap: "./bootstrap.js", input_on_canvas: "./canvasconsole/init_input_on_canvas.js" } ,
  output: {
    path: path.resolve(__dirname, "dist"),
//    filename: "bootstrap.js",
    filename: "[name].js",
  },
  mode: "development",
  plugins: [
    new CopyWebpackPlugin(['index.html', 'canvasconsole/input_on_canvas.html'])
  ],
};
