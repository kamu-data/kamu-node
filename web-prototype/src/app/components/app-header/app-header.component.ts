import {Component, ElementRef, EventEmitter, Input, OnChanges, Output, SimpleChanges, ViewChild} from "@angular/core";
import {Observable, OperatorFunction} from "rxjs";
import {debounceTime, distinctUntilChanged, map} from "rxjs/operators";
import {DatasetIDsInterface, TypeNames} from "../../interface/search.interface";
import {NgbTypeaheadSelectItemEvent} from "@ng-bootstrap/ng-bootstrap";

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

    @Output() public onSelectDataset: EventEmitter<DatasetIDsInterface> = new EventEmitter();
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
    public search: OperatorFunction<string, readonly DatasetIDsInterface[]> = (text$: Observable<string>) => {
        return text$.pipe(
            distinctUntilChanged(),
            map(term => this.ngTypeaheadList ? this.ngTypeaheadList.map((item: DatasetIDsInterface) => item) : [])
        )
    }

    public onSelectItem(event: NgbTypeaheadSelectItemEvent<DatasetIDsInterface>, searchValue: string): void {
        this.isSearchActive = false;

        if(event.item) {
            this.onSelectDataset.emit(event.item);
        }
    }

    public onSearch(event: FocusEvent): void {
        this.isSearchActive = false;
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
