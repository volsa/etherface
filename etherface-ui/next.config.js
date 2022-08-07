/** @type {import('next').NextConfig} */
module.exports = {
  reactStrictMode: true,
  swcMinify: true,

  env: {
    REST_ADDR: process.env.REST_ADDR
  },

  // Redirect any request from "/" to "/hash"
  async redirects() {
    return [
      {
        source: '/',
        destination: '/hash',
        permanent: true,
      },
    ]
  },

}