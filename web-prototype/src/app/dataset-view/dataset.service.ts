import {Injectable} from "@angular/core";
import {from, Observable, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {
    DatasetCurrentUpstreamDependencies,
    DatasetInfoInterface,
    DatasetKindTypeNames,
    DatasetLinageResponse,
    SearchDatasetByID,
    SearchHistoryInterface,
    SearchOverviewDatasetsInterface
} from "../interface/search.interface";
import {expand, filter, map, mergeMap, switchMap, tap} from "rxjs/operators";

@Injectable()
export class AppDatasetService {
    public searchData: any[] = [];
    private searchChanges$: Subject<string> = new Subject<string>();
    // tslint:disable-next-line: no-any
    private searchDataChanges$: Subject<any[]> = new Subject<any[]>();
   // tslint:disable-next-line: no-any
    private searchDatasetInfoChanges$: Subject<any> = new Subject<any>();
    private datasetTree: string[][] = [];

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
    public get getSearchData(): SearchHistoryInterface[] | SearchOverviewDatasetsInterface[] {
        return this.searchData;
    }
    public get getDatasetTree(): string[][] {
        return this.datasetTree;
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


    public onSearchMetadata(id: string): void {
        // tslint:disable-next-line: no-any
        this.searchApi.onSearchMetadata(id).subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }

    public onSearchLinageDataset(id: string): void {
        this.searchApi.searchLinageDataset(id).pipe(
            tap((result: DatasetLinageResponse) => {
                this.changeDatasetTree(result);
            }),
            switchMap((result: DatasetLinageResponse) => {
                return this.recursive(result.metadata.currentUpstreamDependencies);
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

    private changeDatasetTree(dataset: DatasetLinageResponse) {
        if (dataset.kind === DatasetKindTypeNames.derivative) {
            dataset.metadata.currentUpstreamDependencies
                .forEach((dependencies: DatasetCurrentUpstreamDependencies) => {
                    this.datasetTree.push([dataset.id, dependencies.id]);
                })
        }
    }
    private createDependenciesDerivativeList(dataset: DatasetLinageResponse) {
        return dataset.metadata.currentUpstreamDependencies
            .filter((dependencies: DatasetCurrentUpstreamDependencies) => dependencies.kind === DatasetKindTypeNames.derivative);

    }
}