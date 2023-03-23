// Example webpack config for web workers
const path = require('path');
const { WebpackManifestPlugin } = require('webpack-manifest-plugin');
const CleanWebpackPlugin = require('clean-webpack-plugin').CleanWebpackPlugin;

module.exports = {
  entry: {
    // Replace with:
    workerSt: './node_modules/zeropool-client-js/lib/workerSt.js',
    workerMt: './node_modules/zeropool-client-js/lib/workerMt.js',
    // workerSt: './lib/workerSt.js',
    // workerMt: './lib/workerMt.js',
  },
  output: {
    path: path.join(process.cwd(), 'workers'),
    filename: '[name].[fullhash].js',
    assetModuleFilename: '[name][hash][ext]',
    publicPath: './',
  },
  target: 'webworker',
  mode: 'production',
  plugins: [

    new CleanWebpackPlugin(),
    new WebpackManifestPlugin()
  ]
};
