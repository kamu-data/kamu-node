export type Maybe<T> = T | null;
export type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]?: Maybe<T[SubKey]> };
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & { [SubKey in K]: Maybe<T[SubKey]> };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string;
  String: string;
  Boolean: boolean;
  Int: number;
  Float: number;
  DatasetID: any;
  /**
   * Implement the DateTime<Utc> scalar
   *
   * The input/output is a string in RFC3339 format.
   */
  DateTime: any;
  Sha3256: any;
};

export type BlockRef = {
  __typename?: 'BlockRef';
  name: Scalars['String'];
  blockHash: Scalars['Sha3256'];
};

export type DataSchema = {
  __typename?: 'DataSchema';
  format: DataSchemaFormat;
  content: Scalars['String'];
};

export enum DataSchemaFormat {
  Parquet = 'PARQUET',
  ParquetJson = 'PARQUET_JSON'
}

export type DataSlice = {
  __typename?: 'DataSlice';
  format: DataSliceFormat;
  content: Scalars['String'];
};

export enum DataSliceFormat {
  Json = 'JSON',
  JsonLd = 'JSON_LD',
  JsonSoA = 'JSON_SO_A',
  Csv = 'CSV'
}

export type Dataset = {
  __typename?: 'Dataset';
  datasetId: Scalars['DatasetID'];
  id: Scalars['DatasetID'];
  /** Access to the data of the dataset */
  data: DatasetData;
  /** Access to the metadata of the dataset */
  metadata: DatasetMetadata;
  /** Creation time of the first metadata block in the chain */
  createdAt: Scalars['DateTime'];
  /** Creation time of the most recent metadata block in the chain */
  lastUpdatedAt: Scalars['DateTime'];
};

export type DatasetConnection = {
  __typename?: 'DatasetConnection';
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
  /** A list of edges. */
  edges?: Maybe<Array<Maybe<DatasetEdge>>>;
};

export type DatasetData = {
  __typename?: 'DatasetData';
  datasetId: Scalars['DatasetID'];
  /** Total number of records in this dataset */
  numRecordsTotal: Scalars['Int'];
  /** An estimated size of data on disk not accounting for replication or caching */
  estimatedSize: Scalars['Int'];
  /**
   * Returns the specified number of the latest records in the dataset
   * This is equivalent to the SQL query: `SELECT * FROM dataset ORDER BY event_time DESC LIMIT N`
   */
  tail: DataSlice;
};


export type DatasetDataTailArgs = {
  numRecords?: Maybe<Scalars['Int']>;
  format?: Maybe<DataSliceFormat>;
};

/** An edge in a connection. */
export type DatasetEdge = {
  __typename?: 'DatasetEdge';
  /** The item at the end of the edge */
  node: Dataset;
  /** A cursor for use in pagination */
  cursor: Scalars['String'];
};


export type DatasetMetadata = {
  __typename?: 'DatasetMetadata';
  datasetId: Scalars['DatasetID'];
  /** Access to the temporal metadata chain of the dataset */
  chain: MetadataChain;
  /** Last recorded watermark */
  currentWatermark?: Maybe<Scalars['DateTime']>;
  /** Latest data schema */
  currentSchema: DataSchema;
};


export type DatasetMetadataCurrentSchemaArgs = {
  format?: Maybe<DataSchemaFormat>;
};

export type Datasets = {
  __typename?: 'Datasets';
  /** Returns dataset by its ID */
  byId?: Maybe<Dataset>;
  /** Iterates all datasets */
  all: DatasetConnection;
};


export type DatasetsByIdArgs = {
  id: Scalars['DatasetID'];
};


export type DatasetsAllArgs = {
  after?: Maybe<Scalars['String']>;
  before?: Maybe<Scalars['String']>;
  first?: Maybe<Scalars['Int']>;
  last?: Maybe<Scalars['Int']>;
};


export type MetadataBlock = {
  __typename?: 'MetadataBlock';
  blockHash: Scalars['Sha3256'];
  prevBlockHash?: Maybe<Scalars['Sha3256']>;
  systemTime: Scalars['DateTime'];
  outputWatermark?: Maybe<Scalars['DateTime']>;
};

export type MetadataBlockConnection = {
  __typename?: 'MetadataBlockConnection';
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
  /** A list of edges. */
  edges?: Maybe<Array<Maybe<MetadataBlockEdge>>>;
};

/** An edge in a connection. */
export type MetadataBlockEdge = {
  __typename?: 'MetadataBlockEdge';
  /** The item at the end of the edge */
  node: MetadataBlock;
  /** A cursor for use in pagination */
  cursor: Scalars['String'];
};

export type MetadataChain = {
  __typename?: 'MetadataChain';
  /** Returns all named metadata block references */
  refs: Array<BlockRef>;
  /** Returns a metadata block corresponding to the specified hash */
  blockByHash?: Maybe<MetadataBlock>;
  /** Iterates all metadata blocks starting from the latest one */
  blocks: MetadataBlockConnection;
};


export type MetadataChainBlockByHashArgs = {
  hash: Scalars['Sha3256'];
};


export type MetadataChainBlocksArgs = {
  after?: Maybe<Scalars['String']>;
  before?: Maybe<Scalars['String']>;
  first?: Maybe<Scalars['Int']>;
  last?: Maybe<Scalars['Int']>;
};

/** Information about pagination in a connection */
export type PageInfo = {
  __typename?: 'PageInfo';
  /** When paginating backwards, are there more items? */
  hasPreviousPage: Scalars['Boolean'];
  /** When paginating forwards, are there more items? */
  hasNextPage: Scalars['Boolean'];
  /** When paginating backwards, the cursor to continue. */
  startCursor?: Maybe<Scalars['String']>;
  /** When paginating forwards, the cursor to continue. */
  endCursor?: Maybe<Scalars['String']>;
};

export type Query = {
  __typename?: 'Query';
  /** Returns the version of the GQL API */
  apiVersion: Scalars['String'];
  /** Dataset-related functionality group */
  datasets: Datasets;
  /** Search-related functionality group */
  search: Search;
};

export type Search = {
  __typename?: 'Search';
  /** Perform search across all resources */
  query: SearchQueryResultConnection;
};


export type SearchQueryArgs = {
  after?: Maybe<Scalars['String']>;
  before?: Maybe<Scalars['String']>;
  first?: Maybe<Scalars['Int']>;
  last?: Maybe<Scalars['Int']>;
  query: Scalars['String'];
};

export type SearchQueryResult = Dataset;

export type SearchQueryResultConnection = {
  __typename?: 'SearchQueryResultConnection';
  /** Information to aid in pagination. */
  pageInfo: PageInfo;
  /** A list of edges. */
  edges?: Maybe<Array<Maybe<SearchQueryResultEdge>>>;
};

/** An edge in a connection. */
export type SearchQueryResultEdge = {
  __typename?: 'SearchQueryResultEdge';
  /** The item at the end of the edge */
  node: SearchQueryResult;
  /** A cursor for use in pagination */
  cursor: Scalars['String'];
};

