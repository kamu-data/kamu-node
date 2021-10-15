import {Injectable} from "@angular/core";
import {Observable, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {
    SearchHistoryInterface,
    SearchOverviewDatasetsInterface,
    SearchOverviewInterface
} from "../interface/search.interface";

@Injectable()
export class AppDatasetService {
    public searchData: any[] = [];
    private searchChanges$: Subject<string> = new Subject<string>();
    private searchDataChanges$: Subject<any[]> = new Subject<any[]>();

    constructor(
        private searchApi: SearchApi
    ) { }

    public searchChanges(searchValue: string): void {
        this.searchChanges$.next(searchValue);
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
    public searchDataset(id: string): void {
        this.searchApi.searchDataset(id).subscribe((data: SearchHistoryInterface[]) => {
          this.searchData = data;
          this.searchDataChanges(data);
        })
    }
    public onSearchLinageDataset(id: string): void {
        this.searchApi.searchLinageDataset(id).subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
    public onSearchMetadata(id: string): void {
        this.searchApi.onSearchMetadata(id).subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
}