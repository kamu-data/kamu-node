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
  AccountID: any;
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
  /** Returns the kind of a dataset (Root or Derivative) */
  kind: DatasetKind;
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
  /** A shorthand for `edges { node { ... } }` */
  nodes: Array<Dataset>;
  /** Approximate number of total nodes */
  totalCount?: Maybe<Scalars['Int']>;
  /** Page information */
  pageInfo: PageBasedInfo;
  edges: Array<DatasetEdge>;
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

export type DatasetEdge = {
  __typename?: 'DatasetEdge';
  node: Dataset;
};


export enum DatasetKind {
  Root = 'ROOT',
  Derivative = 'DERIVATIVE'
}

export type DatasetMetadata = {
  __typename?: 'DatasetMetadata';
  datasetId: Scalars['DatasetID'];
  /** Access to the temporal metadata chain of the dataset */
  chain: MetadataChain;
  /** Last recorded watermark */
  currentWatermark?: Maybe<Scalars['DateTime']>;
  /** Latest data schema */
  currentSchema: DataSchema;
  /** Current upstream dependencies of a dataset */
  currentUpstreamDependencies: Array<Dataset>;
};


export type DatasetMetadataCurrentSchemaArgs = {
  format?: Maybe<DataSchemaFormat>;
};

export type Datasets = {
  __typename?: 'Datasets';
  /** Returns dataset by its ID */
  byId?: Maybe<Dataset>;
  /** Returns datasets belonging to the specified account */
  byAccount: DatasetConnection;
};


export type DatasetsByIdArgs = {
  id: Scalars['DatasetID'];
};


export type DatasetsByAccountArgs = {
  id: Scalars['AccountID'];
  page?: Maybe<Scalars['Int']>;
  perPage?: Scalars['Int'];
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
  /** A shorthand for `edges { node { ... } }` */
  nodes: Array<MetadataBlock>;
  /** Approximate number of total nodes */
  totalCount?: Maybe<Scalars['Int']>;
  /** Page information */
  pageInfo: PageBasedInfo;
  edges: Array<MetadataBlockEdge>;
};

export type MetadataBlockEdge = {
  __typename?: 'MetadataBlockEdge';
  node: MetadataBlock;
};

export type MetadataChain = {
  __typename?: 'MetadataChain';
  /** Returns all named metadata block references */
  refs: Array<BlockRef>;
  /** Returns a metadata block corresponding to the specified hash */
  blockByHash?: Maybe<MetadataBlock>;
  /** Iterates all metadata blocks in the reverse chronological order */
  blocks: MetadataBlockConnection;
};


export type MetadataChainBlockByHashArgs = {
  hash: Scalars['Sha3256'];
};


export type MetadataChainBlocksArgs = {
  page?: Maybe<Scalars['Int']>;
  perPage?: Scalars['Int'];
};

export type PageBasedInfo = {
  __typename?: 'PageBasedInfo';
  /** When paginating backwards, are there more items? */
  hasPreviousPage: Scalars['Boolean'];
  /** When paginating forwards, are there more items? */
  hasNextPage: Scalars['Boolean'];
  /** Approximate number of total pages assuming number of nodes per page stays the same */
  totalPages?: Maybe<Scalars['Int']>;
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
  query: SearchResultConnection;
};


export type SearchQueryArgs = {
  query: Scalars['String'];
  page?: Maybe<Scalars['Int']>;
  perPage?: Scalars['Int'];
};

export type SearchResult = Dataset;

export type SearchResultConnection = {
  __typename?: 'SearchResultConnection';
  /** A shorthand for `edges { node { ... } }` */
  nodes: Array<SearchResult>;
  /** Approximate number of total nodes */
  totalCount?: Maybe<Scalars['Int']>;
  /** Page information */
  pageInfo: PageBasedInfo;
  edges: Array<SearchResultEdge>;
};

export type SearchResultEdge = {
  __typename?: 'SearchResultEdge';
  node: SearchResult;
};

