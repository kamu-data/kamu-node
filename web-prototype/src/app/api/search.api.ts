import {Injectable} from "@angular/core";
import {Apollo, ApolloBase} from "apollo-angular";
import {map} from "rxjs/operators";
import {ApolloQueryResult, DocumentNode, gql} from "@apollo/client/core";
import {Observable} from "rxjs";
import {
    SearchHistoryInterface,
    SearchOverviewInterface,
} from "../interface/search.interface";

@Injectable()
export class SearchApi {
    private apollo: ApolloBase<any>;

    constructor(private apolloProvider: Apollo) {
        this.apollo = this.apolloProvider.use('newClientName');
    }

    public seatchIndex(): Observable<any> {
        const GET_DATA: DocumentNode = gql``

        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: any) => {
                if (result.data) {
                    return result.data.search.query.edges.map((edge: any) => {
                        let d = Object();
                        d.id = edge.node.id;
                        return d;
                    })
                }
            }));
    }
    public searchOverview(searchQuery: string): Observable<SearchOverviewInterface[]> {
        const GET_DATA: DocumentNode = gql`
  {
  search {
    query(query: "${searchQuery}") {
      edges {
        node {
          __typename
          ... on Dataset {
            id
            kind
            createdAt
            lastUpdatedAt
            __typename
          }
        }
        __typename
      }
      __typename
    }
    __typename
  }
}
`;

        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: any) => {
                if (result.data) {
                    return result.data.search.query.edges.map((edge: any) => {
                        return this.clearlyData(edge);
                    })
                }
            }));
    }

    public searchLinageDataset(id: string): Observable<any> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
    byId(id: "${id}") {
      id
      kind
      metadata {
        currentUpstreamDependencies {
          id
          kind
          __typename
        }
        __typename
      }
      __typename
    }
    __typename
  }
}
`;
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    return result.data;
                }
            }));
    }

    public searchDataset(id: string): Observable<SearchHistoryInterface[]> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
  byId(id: "${id}") {
    id
    createdAt
    lastUpdatedAt
    metadata {
      currentWatermark
      currentSchema(format: "PARQUET_JSON") {
        format
        content
        __typename
      }
      __typename
    }
    data {
      numRecordsTotal
      estimatedSize
      tail(numRecords: 20, format: "JSON") {
        format
        content
        __typename
      }
      __typename
    }
    __typename
  }
  __typename
}

}`
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    return JSON.parse(result.data.datasets.byId.data.tail.content);
                }
            }));
    }

    public onSearchMetadata(id: string): Observable<any> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
    byId(id: "${id}") {
      metadata {
        chain {
          blocks {
            edges {
              node {
                blockHash
                systemTime
                __typename
              }
              __typename
            }
            __typename
          }
          __typename
        }
        __typename
      }
      __typename
    }
    __typename
  }
}
`;
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    debugger
                    return result.data.datasets.byId.metadata.chain.blocks.edges.map((edge: any) => {
                        debugger
                        return this.clearlyData(edge);
                    });
                }
            }));
    }

    clearlyData(edge: any) {
        const object = edge.node;
        const value = 'typename';
        const nodeKeys: string[] = Object.keys(object).filter(key => !key.includes(value));
        let d = Object();

        nodeKeys.forEach((nodeKey: string) => {
            d[nodeKey] = edge.node[nodeKey];
        })

        return d;
    }

}
