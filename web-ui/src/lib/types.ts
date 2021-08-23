export interface Dataset {
    id: String,
    createdAt: Date,
    lastUpdatedAt: Date,
    numRecordsTotal: bigint,
    lastWatermark: Date,
    dataSize: bigint,
}

export interface DatasetViewContext {
    dataset_id: String;
    account_id: String;
}