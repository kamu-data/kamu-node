// This file can be replaced during build by using the `fileReplacements` array.
// `ng build` replaces `environment.ts` with `environment.prod.ts`.
// The list of file replacements can be found in `angular.json`.

export const environment = {
  production: false,
  github_client_id: 'ce6fe6ce924979f88d95', // your Client ID from GitHub
  github_client_secret_id: '4b56ff98415722f4290052bad8ce2ea28d8fe5e2',
  redirect_uri: 'http://127.0.0.1:4200/github_callback', // authentication url
  gatekeeper: 'https://github.com/login/oauth/authorize' // url from gatekeeper
};

/*
 * For easier debugging in development mode, you can import the following file
 * to ignore zone related error stack frames such as `zone.run`, `zoneDelegate.invokeTask`.
 *
 * This import should be commented out in production mode because it will have a negative impact
 * on performance if an error is thrown.
 */
// import 'zone.js/plugins/zone-error';  // Included with Angular CLI.
