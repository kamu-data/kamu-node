import {Component, ElementRef, EventEmitter, Input, Output, ViewChild} from "@angular/core";
import {Observable, OperatorFunction} from "rxjs";
import {debounceTime, distinctUntilChanged, map} from "rxjs/operators";
import {DatasetIDsInterface} from "../../interface/search.interface";

@Component({
  selector: 'app-header',
  templateUrl: './app-header.component.html'
})
export class AppHeaderComponent {
    @Input() public searchValue: string;
    @Input() public appLogo: string;
    @Input() public isMobileView: boolean;
    @Input() public isVisible: boolean;
    @Input() public ngTypeaheadList: DatasetIDsInterface[] = [];

    @Output() public onInputSearch: EventEmitter<string> = new EventEmitter();
    @Output() public keyUpSearchEvent: EventEmitter<string> = new EventEmitter();
    @Output() public addNew: EventEmitter<null> = new EventEmitter();
    @Output() public userInfo: EventEmitter<null> = new EventEmitter();

    @ViewChild('appHeaderMenuButton') appHeaderMenuButton: ElementRef<HTMLElement>;

    public isSearchActive: boolean = false;

    constructor() {}

    public search: OperatorFunction<string, readonly string[]> = (text$: Observable<string>) => {
        return text$.pipe(
            debounceTime(200),
            distinctUntilChanged(),
            map(term => this.ngTypeaheadList.map((item: DatasetIDsInterface) => item.id))
        )
    }


    public onSearch(event: FocusEvent, value: string): void {

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