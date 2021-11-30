import {Injectable} from "@angular/core";
import {from, Observable, pipe, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {
    DatasetCurrentUpstreamDependencies,
    DatasetInfoInterface, DatasetKindInterface,
    DatasetKindTypeNames,
    DatasetLinageResponse,
    SearchDatasetByID,
    SearchHistoryInterface,
    SearchOverviewDatasetsInterface, SearchOverviewInterface
} from "../interface/search.interface";
import {distinct, filter, map, mergeMap, switchMap, tap} from "rxjs/operators";
import {subscribe} from "graphql";

@Injectable()
export class AppDatasetService {
    /* eslint-disable  @typescript-eslint/no-explicit-any */
    public searchData: any[] = [];
    private kindInfoChanges$: Subject<DatasetKindInterface[]> = new Subject<DatasetKindInterface[]>();
    private searchChanges$: Subject<string> = new Subject<string>();
    /* eslint-disable  @typescript-eslint/no-explicit-any */
    private searchDataChanges$: Subject<any[]> = new Subject<any[]>();
    /* eslint-disable  @typescript-eslint/no-explicit-any */
    private searchDatasetInfoChanges$: Subject<any> = new Subject<any>();
    private searchMetadataChanges$: Subject<SearchOverviewInterface> = new Subject<SearchOverviewInterface>();
    private datasetTreeChanges$: Subject<string[][]> = new Subject<string[][]>();
    private datasetTree: string[][] = [];
    private datasetKindInfo: DatasetKindInterface[] = [];

    constructor(
        private searchApi: SearchApi
    ) { }

    public searchDatasetInfoChanges(searchDatasetInfo: DatasetInfoInterface): void {
        this.searchDatasetInfoChanges$.next(searchDatasetInfo);
    }
    public get onSearchDatasetInfoChanges(): Observable<DatasetInfoInterface> {
       return this.searchDatasetInfoChanges$.asObservable();
    }
    public get onSearchChanges(): Observable<string> {
       return this.searchChanges$.asObservable();
    }
    public searchDataChanges(searchData: SearchHistoryInterface[]| SearchOverviewDatasetsInterface[]): void {
        this.searchDataChanges$.next(searchData);
    }
    public get onSearchDataChanges(): Observable<SearchHistoryInterface[] | SearchOverviewDatasetsInterface[]> {
       return this.searchDataChanges$.asObservable();
    }
    public get onKindInfoChanges(): Observable<DatasetKindInterface[]> {
        return this.kindInfoChanges$.asObservable();
    }
    public kindInfoChanges(datasetList: DatasetKindInterface[]): void {
        this.kindInfoChanges$.next(datasetList);
    }
    public get onSearchMetadataChanges(): Observable<SearchOverviewInterface> {
       return this.searchMetadataChanges$.asObservable();
    }
    public searchMetadataChange(data: SearchOverviewInterface) {
       return this.searchMetadataChanges$.next(data);
    }
    public get getSearchData(): SearchHistoryInterface[] | SearchOverviewDatasetsInterface[] {
        return this.searchData;
    }
    public get onDatasetTreeChanges(): Observable<string[][]> {
        return this.datasetTreeChanges$.asObservable();
    }
    public datasetTreeChange(datasetTree: string[][]): void {
        this.datasetTreeChanges$.next(datasetTree);
    }
    public get getDatasetTree(): string[][] {
        return this.datasetTree;
    }
    public get kindInfo(): DatasetKindInterface[] {
        return this.datasetKindInfo;
    }
    public setKindInfo(dataset: DatasetKindInterface): void {
        debugger
        if (this.datasetKindInfo.some((realDataset: DatasetKindInterface) => realDataset.id === dataset.id)) {
            return;
        }
        this.datasetKindInfo.push({id: dataset.id, kind: dataset.kind});
        this.kindInfoChanges(this.datasetKindInfo);
    }

    public resetDatasetTree(): void {
        this.datasetTree = [];
    }
    public searchDataset(id: string, page: number): void {
        this.searchApi.searchDataset({id, page}).subscribe((byID: SearchDatasetByID) => {
            const datasetInfo = AppDatasetService.getDatasetInfo(byID);
            this.searchDatasetInfoChanges(datasetInfo);
            this.searchData = byID.data.tail.content;
            this.searchDataChanges(byID.data.tail.content);
        });
    }

    private static getDatasetInfo(byID: SearchDatasetByID): DatasetInfoInterface {
        return {
            id: byID.id,
            __typename: byID.__typename,
            createdAt: byID.createdAt,
            lastUpdatedAt: byID.lastUpdatedAt,
            estimatedSize: byID.data.estimatedSize,
            numRecordsTotal: byID.data.numRecordsTotal,
            metadata: byID.metadata
        };
    }


    public onSearchMetadata(id: string, page: number): void {
        /* eslint-disable  @typescript-eslint/no-explicit-any */
        this.searchApi.onSearchMetadata({id, page}).subscribe((data: SearchOverviewInterface) => {
            this.searchData = data.dataset;
            this.searchMetadataChange(data);
        })
    }

    public onSearchLinageDataset(id: string): void {
        this.searchApi.searchLinageDataset(id).pipe(
            tap((result: DatasetLinageResponse) => {
                this.changeDatasetTree(result);
            }),
            switchMap((result: DatasetLinageResponse) => {
                debugger
                if (result.kind === DatasetKindTypeNames.derivative) {
                    return this.recursiveUpstreamDependencies(result.id);
                } else {
                    return this.recursive(result.metadata.currentDownstreamDependencies);
                }
            })
        ).subscribe(() => {
            console.log(this.datasetTree);
        });
    }

    public recursive(datasetCurrentUpstreamDependencies: DatasetCurrentUpstreamDependencies[]): Observable<DatasetCurrentUpstreamDependencies[]> {
        return from(datasetCurrentUpstreamDependencies).pipe(
            filter((currentUpstreamDependencies: DatasetCurrentUpstreamDependencies) => {
                return currentUpstreamDependencies.kind === DatasetKindTypeNames.derivative
            }),
            mergeMap((currentUpstreamDependencies: DatasetCurrentUpstreamDependencies) => {
                return this.searchApi.searchLinageDataset(currentUpstreamDependencies.id).pipe(
                    map((result: DatasetLinageResponse) => {
                        this.changeDatasetTree(result);
                        return result;
                    }),
                    mergeMap((result: DatasetLinageResponse) => {
                        const dependenciesDerivativeList: DatasetCurrentUpstreamDependencies[] = this.createDependenciesDerivativeList(result);
                        return this.recursive(dependenciesDerivativeList);
                    })
                )
            })
        );
    }

    public recursiveUpstreamDependencies(id: string): Observable<DatasetCurrentUpstreamDependencies[]> {
        return this.searchApi.searchLinageDatasetUpstreamDependencies(id).pipe(
                    map((result: DatasetLinageResponse) => {
                        this.changeDatasetTree(result);
                        return result;
                    }),
                    mergeMap((result: DatasetLinageResponse) => {
                        const dependenciesDerivativeList: DatasetCurrentUpstreamDependencies[] = this.createDependenciesRootList(result);
                        return this.recursiveUpstreamDependencies(dependenciesDerivativeList);
                    })
                );
    }

    private changeDatasetTree(dataset: DatasetLinageResponse) {
        if (dataset.metadata.currentUpstreamDependencies) {
            dataset.metadata.currentUpstreamDependencies
                .forEach((dependencies: DatasetCurrentUpstreamDependencies) => {
                    this.datasetTree.push([dataset.id, dependencies.id]);
                    this.setKindInfo(dataset);
                    this.setKindInfo(dependencies);
                });
        }
        if (dataset.metadata.currentDownstreamDependencies) {
            dataset.metadata.currentDownstreamDependencies
                .forEach((dependencies: DatasetCurrentUpstreamDependencies) => {
                    this.datasetTree.push([dataset.id, dependencies.id]);
                    this.setKindInfo(dataset);
                    this.setKindInfo(dependencies);
                });
        }
        this.datasetTree = Array.from(this.uniquedatasetTree(this.datasetTree));
        this.datasetTreeChange(this.datasetTree);
    }
    private uniquedatasetTree(datasetTree: string[][]) {
        return new Map(datasetTree.map((p: string[]) => [p.join(), p])).values();
    }
    private createDependenciesDerivativeList(dataset: DatasetLinageResponse) {
        if (dataset.metadata.currentDownstreamDependencies) {
            return dataset.metadata.currentDownstreamDependencies
                .filter((dependencies: DatasetCurrentUpstreamDependencies) => dependencies.kind === DatasetKindTypeNames.derivative);
        }
        if (dataset.metadata.currentUpstreamDependencies) {
           return dataset.metadata.currentUpstreamDependencies
                .filter((dependencies: DatasetCurrentUpstreamDependencies) => dependencies.kind === DatasetKindTypeNames.derivative);
        }
    }
    private createDependenciesRootList(dataset: DatasetLinageResponse) {
        if (dataset.metadata.currentDownstreamDependencies) {
            return dataset.metadata.currentDownstreamDependencies
                .filter((dependencies: DatasetCurrentUpstreamDependencies) => dependencies.kind === DatasetKindTypeNames.root);
        }
        if (dataset.metadata.currentUpstreamDependencies) {
            return dataset.metadata.currentUpstreamDependencies
                .filter((dependencies: DatasetCurrentUpstreamDependencies) => dependencies.kind === DatasetKindTypeNames.root);
        }
    }
}