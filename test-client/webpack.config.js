const path = require('path');
const webpack = require('webpack');
const CopyPlugin = require('copy-webpack-plugin')
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CleanWebpackPlugin = require('clean-webpack-plugin').CleanWebpackPlugin;

module.exports = {
  mode: 'production',
  devtool: false,
  entry: './src/index.ts',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle-[fullhash].js',
    publicPath: './',
  },
  resolve: {
    extensions: ['.tsx', '.ts', '.js'],
    fallback: {
      'http': require.resolve('stream-http'),
      'https': require.resolve('https-browserify'),
      'crypto': require.resolve('crypto-browserify'),
      'os': require.resolve('os-browserify/browser'),
      'path': require.resolve('path-browserify'),
      'assert': require.resolve('assert'),
      'constants': require.resolve('constants-browserify'),
      'fs': false,
    },
    alias: {
      process: 'process/browser.js',
      stream: 'stream-browserify',
    }
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  plugins: [
    new CleanWebpackPlugin(),
    new CopyPlugin({
      patterns: [
        { from: '*', context: 'workers' }
      ],
    }),
    new HtmlWebpackPlugin({
      template: 'src/index.html',
      filename: 'index.html',
    }),
    new webpack.ProvidePlugin({
      Buffer: ['buffer', 'Buffer'],
      process: 'process'
    }),
    new webpack.EnvironmentPlugin({
      CONTRACT_ADDRESS: null,
      TOKEN_ADDRESS: null,
      RELAYER_URL: null,
      RPC_URL: null,
      MNENOMIC: null,
    }),
  ],
  ignoreWarnings: [/Failed to parse source map/],
};
