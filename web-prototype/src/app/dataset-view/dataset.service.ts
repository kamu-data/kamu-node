import {Injectable} from "@angular/core";
import {Observable, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {
    DatasetInfoInterface,
    SearchDatasetByID,
    SearchHistoryInterface,
    SearchOverviewDatasetsInterface
} from "../interface/search.interface";

@Injectable()
export class AppDatasetService {
    public searchData: SearchHistoryInterface[] = [];
    private searchChanges$: Subject<string> = new Subject<string>();
    // tslint:disable-next-line: no-any
    private searchDataChanges$: Subject<any[]> = new Subject<any[]>();
   // tslint:disable-next-line: no-any
    private searchDatasetInfoChanges$: Subject<any> = new Subject<any>();

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


    public onSearchLinageDataset(id: string): void {
        // tslint:disable-next-line: no-any
        this.searchApi.searchLinageDataset(id).subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
    public onSearchMetadata(id: string): void {
        // tslint:disable-next-line: no-any
        this.searchApi.onSearchMetadata(id).subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
}