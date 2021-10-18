import {Component, ElementRef, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild} from "@angular/core";
import {Observable, OperatorFunction} from "rxjs";
import {debounceTime, distinctUntilChanged, map} from "rxjs/operators";
import {DatasetIDsInterface, TypeNames} from "../../interface/search.interface";

@Component({
  selector: 'app-header',
  templateUrl: './app-header.component.html'
})
export class AppHeaderComponent {
    @Input() public searchValue: string = '';
    @Input() public appLogo: string;
    @Input() public isMobileView: boolean;
    @Input() public isVisible: boolean;
    @Input() public ngTypeaheadList: DatasetIDsInterface[] = [];

    @Output() public onInputSearch: EventEmitter<string> = new EventEmitter();
    @Output() public onSelectDataset: EventEmitter<string> = new EventEmitter();
    @Output() public keyUpSearchEvent: EventEmitter<string> = new EventEmitter();
    @Output() public addNew: EventEmitter<null> = new EventEmitter();
    @Output() public userInfo: EventEmitter<null> = new EventEmitter();

    @ViewChild('appHeaderMenuButton') appHeaderMenuButton: ElementRef<HTMLElement>;

    public isSearchActive: boolean = false;
    private _window: Window;

    constructor() {
        this._window = window;
    }

    public isDatasetType(type: string): boolean {
        return type === TypeNames.datasetType;
    }
    public getSearchValue(): string {
        console.log(this._window.location.search.split('?id=')[1].split('&')[0]);
        // return this._window.location.search.split('?id=')[1].split('&')[0] || this.searchValue;
        return this.searchValue;
    }
    public search: OperatorFunction<string, readonly DatasetIDsInterface[]> = (text$: Observable<string>) => {
        return text$.pipe(
            debounceTime(200),
            distinctUntilChanged(),
            map(term => this.ngTypeaheadList.map((item: DatasetIDsInterface) => item))
        )
    }

    public onSelectItem(event: { item: DatasetIDsInterface, preventDefault: () => {} }, searchValue: string): void {
        this.isSearchActive = false;

        if(event.item) {
            if (event.item.__typename === TypeNames.datasetType) {
                this.onSelectDataset.emit(event.item.id);
            } else {
                debugger
                this.onInputSearch.emit(searchValue);
            }
        }
    }

    public onSearch(event: FocusEvent, value: string): void {
        debugger

        this.isSearchActive = false;
        this.onInputSearch.emit(value);
        setTimeout(() => {
            if(this.isMobileView) {
                this.triggerMenuClick();
            }

            (event.target as HTMLElement).blur();
        }, 200)
    }

    // tslint:disable-next-line: no-any
    public onKeyUpSearch(event: any): void {
        this.keyUpSearchEvent.emit(event['target']['value']);
    }
    public onAddNew(): void {
        this.addNew.emit();
    }
    public onOpenUserInfo(): void {
        this.userInfo.emit();
    }

    public triggerMenuClick(): void {
        const el: HTMLElement = this.appHeaderMenuButton.nativeElement;
        el.focus();
        el.click();
        el.blur();
    }
}
