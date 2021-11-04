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

export interface SearchHistoryCurrentSchema {
    name: string,
    type: string,
    fields: [{
        name: string,
        repetition: string,
        type: string,
        logicalType: string
    }]
}

export interface SearchOverviewDatasetsInterface {
    createdAt: string;
    id: string;
    kind: string;
    lastUpdatedAt: string;
}

export interface SearchOverviewInterface {
    dataset: SearchOverviewDatasetsInterface[],
    totalCount: number,
    pageInfo: PageInfoInterface,
    currentPage: number
}

export interface PageInfoInterface {
    hasNextPage: boolean,
    hasPreviousPage: boolean,
    totalPages: number
}

export interface DatasetIDsInterface {
    id: string,
    __typename: TypeNames
}

export enum TypeNames {
    allDataType = 'all',
    datasetType = 'Dataset'
}

export interface SearchDataset {
    datasets: {
        __typename: string,
        byId: SearchDatasetByID
    }
}

export interface SearchDatasetByID {
    __typename: string,
    createdAt: string,
    data: SearchDatasetByIDDataInterface,
    // ca.covid19.daily-cases
    id: string,
    lastUpdatedAt: string,
    metadata: {
        _typename: string,
        currentSchema: {
            _typename: string,
            content: SearchHistoryCurrentSchema[],
            format: string
        },
        currentWatermark: string
    }
}
export interface SearchMetadataNodeResponseInterface {
    blockHash: string,
    systemTime: string
}
export interface SearchDatasetByIDDataInterface {
    __typename: string,
    estimatedSize: number,
    numRecordsTotal: number,
    tail: {
        __typename: string,
        content: SearchHistoryInterface[],
        format: string
    }
}
export interface DatasetInfoInterface {
    __typename: string,
    createdAt: string,
    id: string,
    lastUpdatedAt: string,
    estimatedSize: number,
    numRecordsTotal: number,
    metadata: {
        _typename: string,
        currentSchema: {
            _typename: string,
            content: SearchHistoryCurrentSchema[],
            format: string
        },
        currentWatermark: string
    }
}
export interface DatasetLinageResponse {
    "__typename": string,
    "id": string,
    "kind": DatasetKindTypeNames,
    "metadata": {
        "__typename": string,
        "currentUpstreamDependencies": DatasetCurrentUpstreamDependencies[]
    }
}
export interface DatasetCurrentUpstreamDependencies {
    "__typename": string,
    "id": string,
    "kind": DatasetKindTypeNames
}
export enum DatasetKindTypeNames {
    derivative = 'DERIVATIVE',
    root = 'ROOT'
}