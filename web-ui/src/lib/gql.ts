import { browser } from '$app/env';
import { ApolloClient, HttpLink, InMemoryCache, gql } from '@apollo/client/core';
import type { NormalizedCacheObject } from '@apollo/client';
export { gql } from '@apollo/client/core';

var client: ApolloClient<NormalizedCacheObject> = null;

function createClient() {
    let url = <string>(
        browser
            ? import.meta.env.VITE_API_SERVER_URL_BROWSER ?? import.meta.env.VITE_API_SERVER_URL
            : import.meta.env.VITE_API_SERVER_URL
    );

    const link = new HttpLink({
        uri: url
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
