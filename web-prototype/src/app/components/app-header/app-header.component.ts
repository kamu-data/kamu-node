import {Component, ElementRef, EventEmitter, Input, Output, ViewChild} from "@angular/core";
import {Observable, OperatorFunction} from "rxjs";
import {debounceTime, distinctUntilChanged, switchMap} from "rxjs/operators";
import {DatasetIDsInterface, TypeNames} from "../../interface/search.interface";
import {NgbTypeaheadSelectItemEvent} from "@ng-bootstrap/ng-bootstrap";
import {SearchApi} from "../../api/search.api";
import {UserInterface} from "../../interface/auth.interface";

@Component({
  selector: 'app-header',
  templateUrl: './app-header.component.html'
})
export class AppHeaderComponent {
    @Input() public searchValue: DatasetIDsInterface = {id: '', __typename: TypeNames.allDataType};
    @Input() public appLogo: string;
    @Input() public isMobileView: boolean;
    @Input() public isVisible: boolean;
    @Input() public user: UserInterface;

    @Output() public onSelectDataset: EventEmitter<DatasetIDsInterface> = new EventEmitter();
    @Output() public addNew: EventEmitter<null> = new EventEmitter();
    @Output() public userInfo: EventEmitter<null> = new EventEmitter();

    @ViewChild('appHeaderMenuButton') appHeaderMenuButton: ElementRef<HTMLElement>;

    public isSearchActive = false;
    private _window: Window;

    constructor(private appSearchAPI: SearchApi) {
        this._window = window;
    }

    public isDatasetType(type: string): boolean {
        return type === TypeNames.datasetType;
    }
    public search: OperatorFunction<string, readonly DatasetIDsInterface[]> = (text$: Observable<string>) => {
        return text$.pipe(
            debounceTime(300),
            distinctUntilChanged(),
            switchMap(term => this.appSearchAPI.autocompleteDatasetSearch(term)))
    }

    public formatter(x: DatasetIDsInterface | string): string {
        return typeof x !== 'string' ? x.id : x;
    }

    public onSelectItem(event: any): void {
        this.isSearchActive = false;

        if(event.item) {
            this.onSelectDataset.emit(event.item);
        }
    }

    public onSearch(event: any, searchValue: DatasetIDsInterface | string): void {
        this.isSearchActive = false;

        // eslint-disable-next-line @typescript-eslint/ban-ts-comment
        // @ts-ignore
        if (event["key"] === 'Enter') {
            let searchInfo: DatasetIDsInterface;

            if (typeof searchValue === 'string') {
                searchInfo = {id: searchValue, __typename: TypeNames.allDataType};
            } else searchInfo = searchValue;

            this.onSelectDataset.emit(searchInfo);
        }
        setTimeout(() => {
            if(this.isMobileView) {
                this.triggerMenuClick();
            }

            (event.target as HTMLElement).blur();
            const typeaheadInput: Element | null = document.querySelector("ngb-typeahead-window");
            if (typeaheadInput) {
                // eslint-disable-next-line @typescript-eslint/ban-ts-comment
                // @ts-ignore
                document.querySelector("ngb-typeahead-window").classList.remove('show');
            }
        }, 200);
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
