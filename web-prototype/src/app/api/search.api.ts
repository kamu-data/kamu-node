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

    public searchOverview(): Observable<SearchOverviewInterface[]> {
        const GET_DATA: DocumentNode = gql`
  {
  search {
    query(query: "ca") {
      edges{
        node{
          __typename
          ... on Dataset {
            id
          }
        }
      }
    }
  }
}
`;

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

    public searchLastTenFields(): Observable<any> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
    all(last: 10){
      edges{
        node {
          metadata {
            datasetId
          }
          id,
          kind,
          data {
              tail {
                format
                content
              }
            }
        }
      }
    }
  }
}
`;
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    debugger
                    return result.data;
                }
            }));
    }

    public searchHistory(): Observable<SearchHistoryInterface[]> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
    byId(id: "ca.covid19.daily-cases") {
    	data{
        tail(numRecords: 5, format: JSON){
          content
        }
      }
    }
  }
}`
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    return JSON.parse(result.data.datasets.byId.data.tail.content);
                }
            }));
    }

    public onSearchMetadata(): Observable<any> {
        const GET_DATA: DocumentNode = gql`
{
  datasets {
    all(last: 10){
      edges{
        node {
          metadata {
            datasetId
          }
        }
      }
    }
  }
}
`;
        return this.apollo.watchQuery({query: GET_DATA})
            .valueChanges.pipe(map((result: ApolloQueryResult<any>) => {
                if (result.data) {
                    debugger
                    return result.data;
                }
            }));
    }

}
