/////////////////////////////////////////////////////////////////////////////////////////
// GQL
/////////////////////////////////////////////////////////////////////////////////////////

export interface Dataset {
    id: string,
    createdAt?: Date,
    lastUpdatedAt?: Date,
    metadata?: DatasetMetadata,
    data?: DatasetData,
}

export interface DatasetMetadata {
    currentWatermark?: Date,
    currentSchema?: DatasetSchema,
}

export interface DatasetData {
    numRecordsTotal?: bigint,
    estimatedSize?: bigint,
    tail?: DataSlice,
}

export interface DatasetSchema {
    format: string,
    content: object,
}

export interface DataSlice {
    format: string,
    content: object,
}

/////////////////////////////////////////////////////////////////////////////////////////

export interface DatasetViewContext {
    dataset_id: string;
    account_id: string;
}