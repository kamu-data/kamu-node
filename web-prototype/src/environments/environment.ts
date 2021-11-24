// This file can be replaced during build by using the `fileReplacements` array.
// `ng build` replaces `environment.ts` with `environment.prod.ts`.
// The list of file replacements can be found in `angular.json`.

export const environment = {
  production: false,
  github_client_id: '361a3b4fda86d0234d2f', // your Client ID from GitHub
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
