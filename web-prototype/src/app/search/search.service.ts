import {Injectable} from "@angular/core";
import {Observable, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {
    SearchHistoryInterface,
    SearchOverviewDatasetsInterface,
    SearchOverviewInterface
} from "../interface/search.interface";

@Injectable()
export class AppSearchService {
    public searchData: SearchOverviewInterface;
    private searchChanges$: Subject<string> = new Subject<string>();
    private searchDataChanges$: Subject<SearchOverviewInterface> = new Subject<SearchOverviewInterface>();

    constructor(
        private searchApi: SearchApi
    ) { }

    public searchChanges(searchValue: string): void {
        this.searchChanges$.next(searchValue);
    }
    public get onSearchChanges(): Observable<string> {
       return this.searchChanges$.asObservable();
    }
    public searchDataChanges(searchData: SearchOverviewInterface): void {
        this.searchDataChanges$.next(searchData);
    }
    public get onSearchDataChanges(): Observable<SearchOverviewInterface> {
       return this.searchDataChanges$.asObservable();
    }
    public search(searchValue: string): void {
        this.searchApi.searchOverview(searchValue).subscribe((data: SearchOverviewInterface) => {
            debugger
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
}