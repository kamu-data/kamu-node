export interface Dataset {
    id: string,
    createdAt: Date,
    lastUpdatedAt: Date,
    numRecordsTotal: bigint,
    currentWatermark: Date,
    dataSize: bigint,
    currentSchema: DatasetSchema,
    tail: DataSlice,
}

export interface DatasetViewContext {
    dataset_id: string;
    account_id: string;
}

export interface DatasetSchema {
    format: string,
    content: object,
}

export interface DataSlice {
    format: string,
    content: object,
}