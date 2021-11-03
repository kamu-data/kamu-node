import {Injectable} from "@angular/core";
import {Observable, Subject} from "rxjs";
import {SearchApi} from "../api/search.api";
import {
    DatasetIDsInterface,
    SearchOverviewInterface
} from "../interface/search.interface";

@Injectable()
export class AppSearchService {
    public searchData: SearchOverviewInterface;
    private searchChanges$: Subject<string> = new Subject<string>();
    private searchDataChanges$: Subject<SearchOverviewInterface> = new Subject<SearchOverviewInterface>();
    private autocompleteDatasetChanges$: Subject<DatasetIDsInterface[]> = new Subject<DatasetIDsInterface[]>();

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
    public autocompleteDatasetChanges(searchData: DatasetIDsInterface[]) {
        this.autocompleteDatasetChanges$.next(searchData);
    }
    public get onAutocompleteDatasetChanges(): Observable<DatasetIDsInterface[]> {
       return this.autocompleteDatasetChanges$.asObservable();
    }
    public search(searchValue: string, page = 0): void {
        this.searchApi.searchOverview(searchValue, page).subscribe((data: SearchOverviewInterface) => {
            this.searchData = data;
            this.searchDataChanges(data);
        })
    }
    public autocompleteDatasetSearch(search: string): void {
        this.searchApi.autocompleteDatasetSearch(search).subscribe((data: DatasetIDsInterface[]) => {
            this.autocompleteDatasetChanges(data);
        }, () => {
            this.autocompleteDatasetChanges([])
        });
    }
}