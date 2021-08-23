import { ApolloClient, HttpLink, InMemoryCache, gql } from '@apollo/client/core';
import type { NormalizedCacheObject } from '@apollo/client';
export { gql } from '@apollo/client/core';

var client: ApolloClient<NormalizedCacheObject> = null;

function createClient() {
    const link = new HttpLink({
        uri: 'http://localhost:8080/graphql'
        // headers: {
        // 	Authorization: `Bearer ${authToken}`
        // }
    });

    const cache = new InMemoryCache();

    return new ApolloClient({
        link,
        cache
    });
}

export function getClient() {
    if (client == null) {
        client = createClient();
    }
    return client;
}
