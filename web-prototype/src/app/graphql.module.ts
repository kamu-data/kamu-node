import {NgModule} from '@angular/core';
import {APOLLO_OPTIONS} from 'apollo-angular';
import {InMemoryCache} from '@apollo/client/core';
import {HttpLink, HttpLinkHandler} from 'apollo-angular/http';

const uri = 'kamu'; // <-- add the URL of the GraphQL server here
export function createApollo(httpLink: HttpLink): { cache: InMemoryCache; link: HttpLinkHandler } {
  return {
    link: httpLink.create({uri}),
    cache: new InMemoryCache(),
  };
}

@NgModule({
  providers: [
    {
      provide: APOLLO_OPTIONS,
      useFactory: createApollo,
      deps: [HttpLink],
    },
  ],
})
export class GraphQLModule {}
