export interface SearchHistoryResponseInterface {
    datasets: {
        __typename: string;
        byId: {
            __typename: string;
            data: {
                __typename: string;
                tail: {
                    __typename: string;
                    content: string;
                }
            }
        }
    }
}

export interface SearchResponse {
    data: SearchHistoryResponseInterface,
    loading: boolean,
    networkStatus: number
}

export interface SearchHistoryInterface {
    province: string;
    reported_date: string;
    system_time: string;
    total_daily: number;
}
export interface SearchOverviewInterface {
    id: string
}