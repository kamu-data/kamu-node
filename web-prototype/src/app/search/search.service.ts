import {Injectable} from "@angular/core";
import {Observable, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {SearchHistoryInterface, SearchOverviewInterface} from "../interface/search.interface";

@Injectable()
export class AppSearchService {
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
    public searchDataChanges(searchData: SearchHistoryInterface[]| SearchOverviewInterface[]): void {
        this.searchDataChanges$.next(searchData);
    }
    public get onSearchDataChanges(): Observable<SearchHistoryInterface[] | SearchOverviewInterface[]> {
       return this.searchDataChanges$.asObservable();
    }
    public get getSearchData(): SearchHistoryInterface[] | SearchOverviewInterface[] {
        return this.searchData;
    }
    public searchHistory(): void {
        this.searchApi.searchHistory().subscribe((data: SearchHistoryInterface[]) => {
          this.searchData = data;
          this.searchDataChanges(data);
        })
    }
    public search(searchValue: string): void {
        this.searchApi.searchOverview(searchValue).subscribe((data: SearchOverviewInterface[]) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
    public searchLastTenFields(): void {
        this.searchApi.searchLastTenFields().subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
    public onSearchMetadata(): void {
        this.searchApi.onSearchMetadata().subscribe((data: any) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
}