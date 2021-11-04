import {Injectable} from "@angular/core";
import {Apollo, ApolloBase} from "apollo-angular";
import {map} from "rxjs/operators";
import {ApolloQueryResult, DocumentNode, gql} from "@apollo/client/core";
import {Observable, of} from "rxjs";
import {
    DatasetIDsInterface,
    PageInfoInterface, SearchDatasetByID, SearchMetadataNodeResponseInterface,
    SearchOverviewDatasetsInterface, SearchOverviewInterface, TypeNames,
} from "../interface/search.interface";
import AppValues from "../common/app.values";

@Injectable()
export class SearchApi {
    /* eslint-disable  @typescript-eslint/no-explicit-any */
    private apollo: ApolloBase<any>;

    constructor(private apolloProvider: Apollo) {
        this.apollo = this.apolloProvider.use('newClientName');
    }

    // tslint:disable-next-line: no-any
    public seatchIndex(): Observable<any> {
        const GET_DATA: DocumentNode = gql``

        /* eslint-disable  @typescript-eslint/no-explicit-any */
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: any) => {
                if (result.data) {
                    return result.data.search.query.edges.map((edge: any) => {
                        const d = Object();
                        d.id = edge.node.id;
                        return d;
                    })
                }
            }));
    }
    public searchOverview(searchQuery: string, page = 0): Observable<SearchOverviewInterface> {
        const GET_DATA: DocumentNode = gql`
  {
  search {
    query(query: "${searchQuery}", perPage: 2, page: ${(page).toString()}) {
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
      totalCount
      pageInfo {
        hasNextPage
        hasPreviousPage
        totalPages
      }
      __typename
    }
    __typename
  }
}
`;

        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: any) => {
                let dataset: SearchOverviewDatasetsInterface[] = [];
                let pageInfo: PageInfoInterface = SearchApi.pageInfoInit();
                let totalCount = 0;
                let currentPage = 1;

                if (result.data) {
                    // tslint:disable-next-line: no-any
                    dataset = result.data.search.query.edges.map((edge: any) => {
                        return this.clearlyData(edge.node);
                    })
                    pageInfo = result.data.search.query.pageInfo;
                    totalCount = result.data.search.query.totalCount;
                    currentPage = page;
                }

                return SearchApi.searchOverviewData(dataset, pageInfo, totalCount, currentPage);
            }));
    }
    private static searchOverviewData(dataset: SearchOverviewDatasetsInterface[], pageInfo: PageInfoInterface, totalCount: number, currentPage: number): SearchOverviewInterface {
        return {
            dataset: dataset,
            pageInfo: pageInfo,
            totalCount: totalCount,
            currentPage: currentPage + 1
        };
    }
    private static pageInfoInit(): PageInfoInterface {
        return {
            hasNextPage: false,
            hasPreviousPage: false,
            totalPages: 0
        }
    }
    public autocompleteDatasetSearch(id: string): Observable<DatasetIDsInterface[]> {
        if(id === '') {
            return of([]);
        }
        const GET_DATA: DocumentNode = gql`
{
  search {
    query(query: "${id}", perPage: 10) {
      nodes {
        ... on Dataset {
          id
        }
      }
    }
  }
}`

        /* eslint-disable  @typescript-eslint/no-explicit-any */
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    return SearchApi.searchValueAddToAutocomplete(result.data.search.query.nodes || [], id);
                } else {
                    return [];
                }
            }));
    }
    private static searchValueAddToAutocomplete(ngTypeaheadList: DatasetIDsInterface[], searchValue: string): DatasetIDsInterface[] {
        const newArray: DatasetIDsInterface[] = JSON.parse(JSON.stringify(ngTypeaheadList));
        if (searchValue) {
            newArray.unshift({__typename: TypeNames.allDataType, id: searchValue});
        }
        return newArray;
    }

    /* eslint-disable  @typescript-eslint/no-explicit-any */
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
        }
      }
    }
  }
}
`;
        /* eslint-disable  @typescript-eslint/no-explicit-any */
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    return result.data.datasets.byId;
                }
            }));
    }

    public searchDataset(params: {id: string, numRecords?: number, page?: number}): Observable<SearchDatasetByID> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
  byId(id: "${params.id}") {
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
      tail(numRecords: ${params.numRecords || 10}, format: "JSON") {
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
                    /* eslint-disable  @typescript-eslint/no-explicit-any */
                    const datasets: any = AppValues.deepCopy(result.data.datasets.byId);
                    datasets['data'].tail.content = JSON.parse(result.data.datasets.byId['data'].tail.content);
                    datasets['metadata'].currentSchema.content = JSON.parse(result.data.datasets.byId['metadata'].currentSchema.content);

                    return datasets as SearchDatasetByID;
                }
                /* eslint-disable  @typescript-eslint/no-explicit-any */
                return {} as any;
            }));
    }

    // tslint:disable-next-line: no-any
    public onSearchMetadata(params: {id: string, numRecords?: number, page?: number}): Observable<any> {
        debugger
        const GET_DATA: DocumentNode = gql`
{
  datasets {
    byId(id: "${params.id}") {
      id
      metadata {
        chain {
          blocks(perPage: ${(params.numRecords || 5).toString()}, page: ${(params.page || 0).toString()}) {
            totalCount
            nodes {
              blockHash,
              systemTime
            }
            pageInfo {
              hasNextPage
              hasPreviousPage
              totalPages
            }
          }
        }
      }
    }
  }
}`;
        let datasets: SearchOverviewInterface;

        // tslint:disable-next-line: no-any
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                let dataset: SearchOverviewDatasetsInterface[] = [];
                let pageInfo: PageInfoInterface = SearchApi.pageInfoInit();
                let totalCount = 0;
                let currentPage = params.page || 0;

                if (result.data) {
                    // tslint:disable-next-line: no-any
                    dataset = result.data.datasets.byId.metadata.chain.blocks.nodes.map((node: SearchMetadataNodeResponseInterface) => {
                        return this.clearlyData(node);
                    });
                    pageInfo = result.data.datasets.byId.metadata.chain.blocks.pageInfo;
                    totalCount = result.data.datasets.byId.metadata.chain.blocks.totalCount;
                }

                return SearchApi.searchOverviewData(dataset, pageInfo, totalCount, currentPage);
            }));
    }

    // tslint:disable-next-line: no-any
    clearlyData(edge: any) {
        const object = edge;
        const value = 'typename';
        const nodeKeys: string[] = Object.keys(object).filter(key => !key.includes(value));
        const d = Object();

        nodeKeys.forEach((nodeKey: string) => {
            d[nodeKey] = (edge as any)[nodeKey];
        })

        return d;
    }

}
